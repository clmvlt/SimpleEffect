import type { Flicks, FrameRate } from '@se/protocol';

import { frameDurationFlicks, frameIndexAt, frameStart } from './frame-pacing';
import { nextShuttleRate, type ShuttleDirection } from './shuttle';
import {
  transportPosition,
  type PlaybackBounds,
  type PlaybackSpan,
  type Transport,
} from './transport';

/** The master clock of playback, in seconds (the audio clock in the app). Never goes back. */
export interface PlaybackClock {
  /** The time being presented now (heard, for an audio clock). */
  now(): number;
  /**
   * How far ahead of `now` a sound can still start (audio output latency plus a scheduling
   * margin; 0 by default). Playback is anchored that far ahead, so its first sample is not lost:
   * the picture holds the start frame meanwhile.
   */
  leadTime?(): number;
}

/** Runs a callback before the next display refresh (`requestAnimationFrame` in the app). */
export interface FrameTicker {
  request(callback: () => void): number;
  cancel(handle: number): void;
}

/** What the transport controls show. */
export interface PlaybackState {
  /** 0 when paused; 1 real time; negative in reverse; 2 and 4 fast. */
  readonly rate: number;
  readonly looping: boolean;
}

/** Options of a {@link PlaybackController}. */
export interface PlaybackControllerOptions {
  readonly clock: PlaybackClock;
  readonly frameRate: FrameRate;
  /** Default: `requestAnimationFrame`. */
  readonly ticker?: FrameTicker;
  /** The frame to show changed: `time` is its start (the playhead). */
  readonly onFrame: (time: Flicks) => void;
  /** Rate, anchor or bounds changed: sound must be planned again (`null` when paused). */
  readonly onTransportChange?: (transport: Transport | null, bounds: PlaybackBounds) => void;
  readonly onStateChange?: (state: PlaybackState) => void;
}

const animationFrameTicker: FrameTicker = {
  request: (callback) => requestAnimationFrame(callback),
  cancel: (handle) => {
    cancelAnimationFrame(handle);
  },
};

/**
 * Plays a sequence against a master clock (`docs/architecture.md` §8): play/pause, stop, JKL
 * shuttle, frame steps, looping over a span. On each display refresh it reads the clock, derives
 * the sequence time from the transport anchor and reports the frame to show when it changes, so
 * frames that are late are skipped instead of slowing playback down.
 *
 * The playhead is always the start of the frame on screen. A move made by someone else (the user
 * clicking the ruler) is a `seek`: playback, if running, continues from there.
 */
export class PlaybackController {
  private frameRate: FrameRate;
  private time = 0 as Flicks;
  private runStart = 0 as Flicks;
  private transport: Transport | null = null;
  private contentEnd = 0 as Flicks;
  private loopRange: PlaybackSpan | null = null;
  private looping = false;
  private tickHandle: number | null = null;
  private destroyed = false;
  private readonly ticker: FrameTicker;

  constructor(private readonly options: PlaybackControllerOptions) {
    this.frameRate = options.frameRate;
    this.ticker = options.ticker ?? animationFrameTicker;
  }

  /** The playhead: start of the frame on screen. */
  currentTime(): Flicks {
    return this.time;
  }

  getState(): PlaybackState {
    return { rate: this.transport?.rate ?? 0, looping: this.looping };
  }

  /** The running transport and its bounds (`null` transport when paused). */
  getTransport(): { readonly transport: Transport | null; readonly bounds: PlaybackBounds } {
    return { transport: this.transport, bounds: this.bounds() };
  }

  setFrameRate(rate: FrameRate): void {
    this.frameRate = rate;
  }

  /** End of the sequence's content: forward playback stops there unless looping. */
  setContentEnd(end: Flicks): void {
    if (end === this.contentEnd) return;
    this.reanchor(() => {
      this.contentEnd = end;
    });
  }

  /** The in/out span looped over (`null`: the whole content). */
  setLoopRange(range: PlaybackSpan | null): void {
    if (range?.start === this.loopRange?.start && range?.end === this.loopRange?.end) return;
    this.reanchor(() => {
      this.loopRange = range;
    });
  }

  setLooping(looping: boolean): void {
    if (looping === this.looping) return;
    this.reanchor(() => {
      this.looping = looping;
    });
    this.emitState();
  }

  /** The playhead was moved elsewhere: show `time`, and keep playing from it if playing. */
  seek(time: Flicks): void {
    this.time = time;
    this.runStart = time;
    const { transport } = this;
    if (transport === null) return;
    this.transport = this.anchoredAt(time, transport.rate);
    this.emitTransport();
  }

  /** Plays at `rate` from the playhead (or from the current position when already playing). */
  play(rate = 1): void {
    if (this.destroyed) return;
    if (rate === 0) {
      this.pause();
      return;
    }
    const current = this.transport;
    if (current !== null) {
      const position = transportPosition(current, this.now(), this.bounds());
      this.transport = { rate, anchorTime: position.time, anchorClock: this.anchorClock() };
    } else {
      const start = this.startTime(rate);
      if (start === null) return;
      this.runStart = start;
      this.transport = this.anchoredAt(start, rate);
      this.moveTo(start);
    }
    this.emitTransport();
    this.emitState();
    this.scheduleTick();
  }

  /** Stops on the frame on screen. */
  pause(): void {
    if (this.transport === null) return;
    this.advance();
    this.transport = null;
    this.cancelTick();
    this.emitTransport();
    this.emitState();
  }

  togglePlay(): void {
    if (this.transport === null) this.play(1);
    else this.pause();
  }

  /** Pauses and returns to where this playback started. */
  stop(): void {
    this.pause();
    this.moveTo(this.runStart);
  }

  /** J or L: starts or speeds up playback in that direction. */
  shuttle(direction: ShuttleDirection): void {
    this.play(nextShuttleRate(this.transport?.rate ?? 0, direction));
  }

  /** Pauses and moves the playhead by `frames` (negative = back), within the content. */
  step(frames: number): void {
    this.pause();
    const index = frameIndexAt(this.time, this.frameRate) + frames;
    const last = frameIndexAt(this.contentEnd, this.frameRate);
    this.moveTo(frameStart(Math.max(0, Math.min(last, index)), this.frameRate));
  }

  destroy(): void {
    this.cancelTick();
    this.transport = null;
    this.destroyed = true;
  }

  private bounds(): PlaybackBounds {
    const whole = { start: 0 as Flicks, end: this.contentEnd };
    return this.looping
      ? { span: this.loopRange ?? whole, loop: true }
      : { span: whole, loop: false };
  }

  private now(): number {
    return this.options.clock.now();
  }

  /** Clock time a new anchor takes effect: the earliest time its sound can start. */
  private anchorClock(): number {
    return this.now() + (this.options.clock.leadTime?.() ?? 0);
  }

  /**
   * A transport showing the frame starting at `time` first: reverse playback starts at the end of
   * that frame, otherwise it would show the previous frame at once.
   */
  private anchoredAt(time: Flicks, rate: number): Transport {
    const anchorTime = rate < 0 ? time + frameDurationFlicks(this.frameRate) - 1 : time;
    return { rate, anchorTime, anchorClock: this.anchorClock() };
  }

  /** Where playback at `rate` starts from the playhead, or `null` when there is nothing to play. */
  private startTime(rate: number): Flicks | null {
    const { span, loop } = this.bounds();
    if (span.end <= span.start) return null;
    const lastFrame = frameStart(frameIndexAt(span.end - 1, this.frameRate), this.frameRate);
    const inside = this.time >= span.start && this.time < span.end;
    if (loop && !inside) return rate > 0 ? span.start : lastFrame;
    if (rate > 0) return this.time >= span.end ? span.start : this.time;
    return this.time <= span.start ? null : (Math.min(this.time, lastFrame) as Flicks);
  }

  /** Applies a change of bounds while keeping the playing position. */
  private reanchor(change: () => void): void {
    const current = this.transport;
    if (current === null) {
      change();
      return;
    }
    // The new anchor is where the old transport will be then: the picture does not hold.
    const anchorClock = this.anchorClock();
    const position = transportPosition(current, anchorClock, this.bounds());
    change();
    this.transport = { rate: current.rate, anchorTime: position.time, anchorClock };
    this.emitTransport();
  }

  /** Moves the playhead to the frame the clock says; pauses at the edge of a non-looping span. */
  private advance(): void {
    const transport = this.transport;
    if (transport === null) return;
    const position = transportPosition(transport, this.now(), this.bounds());
    const index = frameIndexAt(position.time, this.frameRate);
    const shown = frameStart(position.ended ? index : Math.max(0, index), this.frameRate);
    if (position.ended) {
      this.transport = null;
      this.cancelTick();
      this.emitTransport();
      this.emitState();
    }
    this.moveTo(shown);
  }

  private moveTo(time: Flicks): void {
    if (time === this.time) return;
    this.time = time;
    this.options.onFrame(time);
  }

  private scheduleTick(): void {
    if (this.tickHandle !== null || this.destroyed) return;
    this.tickHandle = this.ticker.request(() => {
      this.tickHandle = null;
      if (!this.isPlaying()) return;
      this.advance();
      if (this.isPlaying()) this.scheduleTick();
    });
  }

  /** Read through a method: `advance` may pause, which flow analysis cannot see. */
  private isPlaying(): boolean {
    return this.transport !== null;
  }

  private cancelTick(): void {
    if (this.tickHandle === null) return;
    this.ticker.cancel(this.tickHandle);
    this.tickHandle = null;
  }

  private emitTransport(): void {
    this.options.onTransportChange?.(this.transport, this.bounds());
  }

  private emitState(): void {
    this.options.onStateChange?.(this.getState());
  }
}
