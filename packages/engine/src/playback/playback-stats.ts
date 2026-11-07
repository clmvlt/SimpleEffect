import type { FrameRate } from '@se/protocol';

import { frameIndexAt, skippedFrames } from './frame-pacing';

/** What playback achieved, for the performance HUD. */
export interface PlaybackStats {
  /** Distinct frames drawn during the last second. */
  readonly framesPerSecond: number;
  /** Frames skipped because they were late, since playback started. */
  readonly droppedFrames: number;
}

/** Window of the frames-per-second measure, in milliseconds. */
const FPS_WINDOW_MS = 1000;

/**
 * Counts the frames a playback drew: real frames per second over a sliding second and the frames
 * skipped to stay on the clock. Times come from the caller (`performance.now()` in the engine), so
 * tests drive it with a simulated clock.
 */
export class PlaybackStatsCounter {
  private readonly drawnAt: number[] = [];
  private lastIndex: number | null = null;
  private dropped = 0;

  /** Records the frame drawn at sequence `time` while playing at `rate`, at `nowMs`. */
  frameDrawn(time: number, frameRate: FrameRate, rate: number, nowMs: number): void {
    const index = frameIndexAt(time, frameRate);
    if (index === this.lastIndex) return;
    if (this.lastIndex !== null) this.dropped += skippedFrames(this.lastIndex, index, rate);
    this.lastIndex = index;
    this.drawnAt.push(nowMs);
    this.forget(nowMs);
  }

  /** A jump that is not playback (seek, loop restart): the next frame is not compared to it. */
  discontinuity(): void {
    this.lastIndex = null;
  }

  /** Starts counting again (new playback run). */
  reset(): void {
    this.drawnAt.length = 0;
    this.lastIndex = null;
    this.dropped = 0;
  }

  stats(nowMs: number): PlaybackStats {
    this.forget(nowMs);
    return { framesPerSecond: this.drawnAt.length, droppedFrames: this.dropped };
  }

  private forget(nowMs: number): void {
    while (this.drawnAt.length > 0 && (this.drawnAt[0] ?? 0) <= nowMs - FPS_WINDOW_MS) {
      this.drawnAt.shift();
    }
  }
}
