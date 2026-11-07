import type { Flicks, FrameRate, SequenceId } from '@se/protocol';
import type { AudioEngine, AudioEngineStats, MeterLevels } from '@se/engine/audio';
import {
  PlaybackController,
  type FrameTicker,
  type PlaybackClock,
  type PlaybackSpan,
  type PlaybackState,
  type ShuttleDirection,
} from '@se/engine/playback';
import type { MediaLocator, PreviewPlayback } from '@se/engine/preview';

import { setPlaybackRate } from '../../../shared/playback';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../../timeline';
import { AudioUnsupportedError, type AudioEngineFactory } from './audio-engine-factory';
import type { PlaybackScene } from './viewer-scene';

/** Silence on both channels (playback without an audio engine). */
const SILENCE: MeterLevels = {
  peakDb: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
  rmsDb: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
};

const PERFORMANCE_CLOCK: PlaybackClock = { now: () => performance.now() / 1000 };

/** The sequence playback runs on. */
export interface PlaybackTarget {
  readonly sequenceId: SequenceId;
  readonly frameRate: FrameRate;
}

/** Options of a {@link PlaybackSession}. */
export interface PlaybackSessionOptions {
  readonly createAudio: AudioEngineFactory;
  /** The picture engine (the viewer's engine controller). */
  readonly picture: { setPlayback(playback: PreviewPlayback | null): void };
  /** Display refresh (default `requestAnimationFrame`). */
  readonly ticker?: FrameTicker;
  /** Clock without sound (default `performance.now`). */
  readonly fallbackClock?: PlaybackClock;
}

/**
 * The viewer's playback, outside React: a `PlaybackController` on the audio clock, the audio
 * engine (started lazily; without it playback runs silent on the performance clock), the playhead
 * store (playback writes it; a move made elsewhere is a seek, with an audio scrub snippet when
 * paused) and the picture engine (switched to playback decoding).
 */
export class PlaybackSession {
  private readonly controller: PlaybackController;
  private readonly listeners = new Set<() => void>();
  private readonly fallbackClock: PlaybackClock;
  private audio: AudioEngine | null = null;
  private target: PlaybackTarget | null = null;
  private scene: PlaybackScene | null = null;
  private contentEnd = 0 as Flicks;
  private locator: MediaLocator = () => null;
  private state: PlaybackState = { rate: 0, looping: false };
  private scrubAudio = true;
  private unsubscribePlayhead: (() => void) | null = null;
  private running = false;
  private generation = 0;

  constructor(private readonly options: PlaybackSessionOptions) {
    this.fallbackClock = options.fallbackClock ?? PERFORMANCE_CLOCK;
    const clock: PlaybackClock = {
      now: () => this.clock().now(),
      leadTime: () => this.clock().leadTime?.() ?? 0,
    };
    this.controller = new PlaybackController({
      clock,
      frameRate: { numerator: 30, denominator: 1 },
      ...(options.ticker === undefined ? {} : { ticker: options.ticker }),
      onFrame: (time) => {
        if (this.target !== null) setPlayheadTime(this.target.sequenceId, time);
      },
      onTransportChange: (transport, bounds) => {
        this.audio?.setTransport(transport, bounds);
        this.options.picture.setPlayback(this.picturePlayback(transport?.rate ?? 0));
      },
      onStateChange: (state) => {
        this.state = state;
        setPlaybackRate(state.rate);
        for (const listener of this.listeners) listener();
      },
    });
  }

  /** Starts the audio engine and follows the playhead (React mounts may start it again later). */
  start(): void {
    if (this.running) return;
    this.running = true;
    this.generation += 1;
    const generation = this.generation;
    this.followPlayhead();
    const locator: MediaLocator = (assetId, variant) => this.locator(assetId, variant);
    this.options.createAudio({ locator }).then(
      (engine) => {
        if (generation === this.generation) this.adoptAudio(engine);
        else engine.destroy();
      },
      (error: unknown) => {
        // Without Web Audio (tests) playback is silent by design; anything else is worth a trace.
        if (!(error instanceof AudioUnsupportedError)) {
          console.warn('[viewer] the audio engine could not start: playback is silent', error);
        }
      },
    );
  }

  /** Pauses, closes the audio engine and stops following the playhead; `start` resumes. */
  stop(): void {
    if (!this.running) return;
    this.running = false;
    this.generation += 1;
    this.controller.pause();
    this.unsubscribePlayhead?.();
    this.unsubscribePlayhead = null;
    this.audio?.destroy();
    this.audio = null;
    this.options.picture.setPlayback(null);
    setPlaybackRate(0);
  }

  /** Shows `target` (the active sequence): playback pauses when it changes. */
  setTarget(target: PlaybackTarget): void {
    const current = this.target;
    if (
      current?.sequenceId === target.sequenceId &&
      sameRate(current.frameRate, target.frameRate)
    ) {
      return;
    }
    this.controller.pause();
    this.target = target;
    this.controller.setFrameRate(target.frameRate);
    this.controller.seek(playheadTimeOf(playheadStore.getState(), target.sequenceId));
    if (this.running) this.followPlayhead();
  }

  /** The scene played and where its content ends. */
  setScene(scene: PlaybackScene | null, contentEnd: Flicks): void {
    this.scene = scene;
    this.contentEnd = contentEnd;
    this.audio?.setScene(scene);
    this.controller.setContentEnd(contentEnd);
  }

  setLocator(locator: MediaLocator): void {
    this.locator = locator;
    this.audio?.setLocator(locator);
  }

  /** The in/out span loops are played over (`null`: the whole content). */
  setLoopRange(span: PlaybackSpan | null): void {
    this.controller.setLoopRange(span);
  }

  setLooping(looping: boolean): void {
    this.controller.setLooping(looping);
  }

  setScrubAudio(enabled: boolean): void {
    this.scrubAudio = enabled;
  }

  togglePlay(): void {
    void this.audio?.resume();
    this.controller.togglePlay();
  }

  pause(): void {
    this.controller.pause();
  }

  /** Pauses and returns to where this playback started (the transport's Stop button). */
  stopPlayback(): void {
    this.controller.stop();
  }

  /** J (reverse) or L (forward): starts or speeds up playback that way. */
  shuttle(direction: ShuttleDirection): void {
    void this.audio?.resume();
    this.controller.shuttle(direction);
  }

  /** Pauses and moves by `frames`, with an audio snippet of the new frame. */
  step(frames: number): void {
    this.controller.step(frames);
    if (this.scrubAudio) this.audio?.scrub(this.controller.currentTime());
  }

  goToStart(): void {
    this.moveTo(0 as Flicks);
  }

  /** To the end of the content (not of the loop span). */
  goToEnd(): void {
    this.moveTo(this.contentEnd);
  }

  getState = (): PlaybackState => this.state;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  /** Master levels in dBFS (silence without an audio engine). */
  levels(): MeterLevels {
    return this.audio?.levels() ?? SILENCE;
  }

  audioStats(): AudioEngineStats | null {
    return this.audio?.stats() ?? null;
  }

  private clock(): PlaybackClock {
    return this.audio?.clock ?? this.fallbackClock;
  }

  /** Follows playhead moves made elsewhere (seeks) on the target sequence. */
  private followPlayhead(): void {
    this.unsubscribePlayhead?.();
    const target = this.target;
    if (target === null) return;
    this.controller.seek(playheadTimeOf(playheadStore.getState(), target.sequenceId));
    this.unsubscribePlayhead = playheadStore.subscribe((state) => {
      this.onPlayheadMove(playheadTimeOf(state, target.sequenceId));
    });
  }

  /** The audio engine is ready: it becomes the clock (between two playbacks only). */
  private adoptAudio(engine: AudioEngine): void {
    this.controller.pause();
    this.audio = engine;
    engine.setLocator(this.locator);
    engine.setScene(this.scene);
  }

  private onPlayheadMove(time: Flicks): void {
    if (time === this.controller.currentTime()) return;
    this.controller.seek(time);
    if (this.state.rate === 0 && this.scrubAudio) this.audio?.scrub(time);
  }

  private moveTo(time: Flicks): void {
    if (this.target !== null) setPlayheadTime(this.target.sequenceId, time);
  }

  private picturePlayback(rate: number): PreviewPlayback | null {
    if (rate === 0 || this.target === null) return null;
    return { rate, frameRate: this.target.frameRate };
  }
}

function sameRate(left: FrameRate, right: FrameRate): boolean {
  return left.numerator === right.numerator && left.denominator === right.denominator;
}
