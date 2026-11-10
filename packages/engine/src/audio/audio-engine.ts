import type { AssetId, Flicks } from '@se/protocol';

import type { MediaLocator } from '../decode/media-variant';
import type { AssetTable } from '../evaluate/item-layer';
import type { PlaybackClock } from '../playback/playback-controller';
import type { PlaybackBounds, Transport } from '../playback/transport';
import { AudioChunkPool, type AudioChunkPoolStats } from './audio-chunk-pool';
import { audioClipsOf, type PlayableSequence } from './audio-clips';
import { AudioGraph } from './audio-graph';
import { AudioScheduler } from './audio-scheduler';
import { soundFileOf } from './sound-file';
import type { AudioReaderFactory } from './audio-source';
import { SILENT_LEVELS, type AudioMeter, type MeterLevels } from './meter/audio-meter';

/** What the audio engine plays: a sequence's tracks and the project's assets. */
export interface AudioScene {
  readonly sequence: PlayableSequence;
  readonly assets: AssetTable;
}

/** Audio counters for the performance HUD. */
export interface AudioEngineStats extends AudioChunkPoolStats {
  /** Sound sources started and not yet ended. */
  readonly voices: number;
}

/**
 * The sound of playback (`docs/architecture.md` §8): the Web Audio graph, chunk decoding,
 * scheduling ahead of the audio clock, audio scrub and the master level meter. Its `clock` is
 * the master clock of the `PlaybackController`.
 */
export interface AudioEngine {
  readonly clock: PlaybackClock;
  setScene(scene: AudioScene | null): void;
  /** New media URLs (restarted core, changed files). */
  setLocator(locator: MediaLocator): void;
  /** Playback state from the controller (sound plays at real-time forward speed only). */
  setTransport(transport: Transport | null, bounds: PlaybackBounds): void;
  /** A short snippet of the sound at `time` (paused playhead moves). */
  scrub(time: Flicks): void;
  /** Starts the audio clock if the context waits for a user gesture. */
  resume(): Promise<void>;
  levels(): MeterLevels;
  stats(): AudioEngineStats;
  /** Stops all sound and closes the audio context. */
  destroy(): void;
}

/** The parts of a {@link createAudioEngineWith} engine. */
export interface AudioEngineParts {
  readonly context: AudioContext;
  readonly clock: PlaybackClock;
  readonly meter: AudioMeter | null;
  readonly locator: MediaLocator;
  readonly createReader: AudioReaderFactory;
  /** Where the master goes (default: the speakers; a tap in the A/V sync test). */
  readonly destination?: AudioNode;
}

/** Interval of the scheduling ticks (well inside the 200 ms anticipation). */
const TICK_MS = 25;

/** Whether `asset` has sound to play: an audio or video file with a sound stream. */
class WebAudioEngine implements AudioEngine {
  readonly clock: PlaybackClock;
  private readonly graph: AudioGraph;
  private readonly pool: AudioChunkPool;
  private readonly scheduler: AudioScheduler;
  private assets: AssetTable = {};
  private locator: MediaLocator;
  private timer: ReturnType<typeof setInterval> | null = null;
  private destroyed = false;

  constructor(private readonly parts: AudioEngineParts) {
    this.clock = parts.clock;
    this.locator = parts.locator;
    this.graph = new AudioGraph(parts.context, parts.destination);
    if (parts.meter !== null) this.graph.tap(parts.meter.node);
    this.pool = new AudioChunkPool({ createReader: parts.createReader });
    this.scheduler = new AudioScheduler(this.graph, this.pool);
    this.pool.setLocator((assetId) => this.fileOf(assetId));
  }

  setScene(scene: AudioScene | null): void {
    if (this.destroyed) return;
    this.assets = scene?.assets ?? {};
    this.graph.setMix(scene?.sequence.tracks ?? []);
    this.scheduler.setClips(scene === null ? [] : audioClipsOf(scene.sequence));
  }

  setLocator(locator: MediaLocator): void {
    this.locator = locator;
  }

  setTransport(transport: Transport | null, bounds: PlaybackBounds): void {
    if (this.destroyed) return;
    this.scheduler.setTransport(transport, bounds);
    this.keepTicking();
  }

  scrub(time: Flicks): void {
    if (this.destroyed) return;
    this.scheduler.scrub(time);
    this.keepTicking();
  }

  async resume(): Promise<void> {
    if (this.parts.context.state === 'suspended') await this.parts.context.resume();
  }

  levels(): MeterLevels {
    return this.parts.meter?.levels() ?? SILENT_LEVELS;
  }

  stats(): AudioEngineStats {
    return { ...this.pool.stats(), voices: this.scheduler.voiceCount };
  }

  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;
    if (this.timer !== null) clearInterval(this.timer);
    this.timer = null;
    this.scheduler.stop();
    this.graph.dispose();
    this.parts.meter?.dispose();
    this.pool.destroy();
    void this.parts.context.close();
  }

  /** The file whose sound `assetId` plays: its original, or its mezzanine (never the proxy). */
  private fileOf(assetId: AssetId): string | null {
    return soundFileOf(this.assets, this.locator, assetId);
  }

  /** Runs scheduling ticks while sound plays or a scrub snippet waits. */
  private keepTicking(): void {
    if (this.timer !== null || !this.scheduler.busy) return;
    this.timer = setInterval(() => {
      this.scheduler.tick();
      if (this.scheduler.busy || this.timer === null) return;
      clearInterval(this.timer);
      this.timer = null;
    }, TICK_MS);
  }
}

/** Assembles an audio engine from its parts (tests and the browser A/V check inject them). */
export function createAudioEngineWith(parts: AudioEngineParts): AudioEngine {
  return new WebAudioEngine(parts);
}
