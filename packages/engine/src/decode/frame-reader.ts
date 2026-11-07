import type { DecodedVideoFrame, VideoFrameSource } from './video-source';

/** A caller waiting for the frame shown at `time`. */
interface FrameWaiter {
  readonly time: number;
  readonly resolve: (delivered: boolean) => void;
}

/**
 * What a reader may hold decoded ahead of the time on screen: frames (hardware decoders stall
 * when too many are held) and bytes (the cache evicts beyond its budget: a 1080p frame is 8 MB).
 */
export interface ReadAheadBudget {
  readonly frames: number;
  readonly bytes: number;
}

/** A frame decoded ahead of the time on screen. */
interface AheadFrame {
  readonly end: number;
  readonly bytes: number;
}

/** Options of a {@link FrameReader}. */
export interface FrameReaderOptions {
  readonly source: VideoFrameSource;
  /** Media seconds where reading starts (the frame shown there comes first). */
  readonly start: number;
  /** Reading pauses once it has decoded past this time (see `extend`). */
  readonly until: number;
  /** What may be decoded ahead of `from` (see `extend`) before reading pauses. */
  readonly budget: ReadAheadBudget;
  /** Receives every decoded frame (ownership included). */
  readonly deliver: (frame: DecodedVideoFrame) => void;
}

/**
 * Decodes one file sequentially with a single decoder (playback), from `start` up to a moving
 * limit, handing each frame to `deliver` (the decoder pool caches it). It never decodes more than
 * its budget past the time on screen, so hardware decoders keep free surfaces and the cache never
 * evicts frames before they are shown. A failure
 * stops it quietly: frame requests then fall back to random access, which reports the error.
 */
export class FrameReader {
  readonly start: number;
  private until: number;
  private from: number;
  private budget: ReadAheadBudget;
  private position: number;
  private readonly ahead: AheadFrame[] = [];
  private aheadBytes = 0;
  private readonly waiters = new Set<FrameWaiter>();
  private wake: (() => void) | null = null;
  private iterator: AsyncGenerator<DecodedVideoFrame, void, undefined> | null = null;
  private active = true;

  constructor(private readonly options: FrameReaderOptions) {
    this.start = options.start;
    this.until = options.until;
    this.from = options.start;
    this.budget = options.budget;
    this.position = options.start;
    void this.run();
  }

  /** End time of the last frame decoded (the start before the first one). */
  get decodedUntil(): number {
    return this.position;
  }

  /** Whether the reader is still decoding or waiting for more room. */
  get running(): boolean {
    return this.active;
  }

  /** Moves the window: the time on screen is `from`, keep decoding up to `until`. */
  extend(from: number, until: number, budget: ReadAheadBudget): void {
    this.from = Math.max(this.from, from);
    this.until = Math.max(this.until, until);
    this.budget = budget;
    for (let first = this.ahead[0]; first !== undefined && first.end <= this.from;) {
      this.ahead.shift();
      this.aheadBytes -= first.bytes;
      first = this.ahead[0];
    }
    this.resume();
  }

  /** Whether the frame shown at `time` is still to come from this reader. */
  willDeliver(time: number): boolean {
    return this.active && time >= this.start && time >= this.position && time < this.until;
  }

  /**
   * Resolves `true` once the frame shown at `time` was delivered, `false` if the reader went past
   * it without one, stopped, or `signal` aborted first.
   */
  waitFor(time: number, signal?: AbortSignal): Promise<boolean> {
    if (!this.willDeliver(time) || signal?.aborted === true) return Promise.resolve(false);
    return new Promise((resolve) => {
      const waiter: FrameWaiter = { time, resolve };
      this.waiters.add(waiter);
      signal?.addEventListener(
        'abort',
        () => {
          this.waiters.delete(waiter);
          resolve(false);
        },
        { once: true },
      );
    });
  }

  /** Stops decoding and frees the decoder. */
  stop(): void {
    if (!this.active) return;
    this.active = false;
    this.resume();
    void this.iterator?.return(undefined);
    this.settleWaiters(Infinity, false);
  }

  private async run(): Promise<void> {
    try {
      this.iterator = this.options.source.frames(this.start);
      while (this.active) {
        if (this.position >= this.until || this.full()) {
          await new Promise<void>((resolve) => (this.wake = resolve));
          continue;
        }
        const next = await this.iterator.next();
        if (next.done === true) break;
        this.accept(next.value);
      }
    } catch {
      // Random access reports the failure when the frame is asked for.
    } finally {
      this.stop();
    }
  }

  private accept(frame: DecodedVideoFrame): void {
    if (!this.active) {
      frame.frame.close();
      return;
    }
    this.position = frame.end;
    if (frame.end > this.from) {
      const bytes = frame.frame.displayWidth * frame.frame.displayHeight * 4;
      this.ahead.push({ end: frame.end, bytes });
      this.aheadBytes += bytes;
    }
    this.options.deliver(frame);
    for (const waiter of this.waiters) {
      if (waiter.time < frame.end) {
        this.waiters.delete(waiter);
        waiter.resolve(waiter.time >= frame.start);
      }
    }
  }

  private full(): boolean {
    return this.ahead.length >= this.budget.frames || this.aheadBytes >= this.budget.bytes;
  }

  private settleWaiters(before: number, delivered: boolean): void {
    for (const waiter of this.waiters) {
      if (waiter.time < before) {
        this.waiters.delete(waiter);
        waiter.resolve(delivered);
      }
    }
  }

  private resume(): void {
    const wake = this.wake;
    this.wake = null;
    wake?.();
  }
}
