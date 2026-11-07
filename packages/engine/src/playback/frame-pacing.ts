import type { Flicks, FrameRate } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../time/flicks';

/** Duration of one frame at `rate`, in flicks (exact for common rates). */
export function frameDurationFlicks(rate: FrameRate): number {
  return (FLICKS_PER_SECOND * rate.denominator) / rate.numerator;
}

/** Index of the frame shown at sequence `time` (frames are half-open: a frame owns its start). */
export function frameIndexAt(time: number, rate: FrameRate): number {
  return Math.floor(time / frameDurationFlicks(rate));
}

/** Start of frame `index`, in whole flicks. */
export function frameStart(index: number, rate: FrameRate): Flicks {
  return Math.round(index * frameDurationFlicks(rate)) as Flicks;
}

/**
 * Frames that should have been shown between two displayed frames but were not (late frames
 * skipped to stay on the clock). At a speed of n× only every n-th frame is expected, so only
 * jumps beyond that count; a jump against the playing direction (a loop wrap, a seek) counts none.
 */
export function skippedFrames(previousIndex: number, nextIndex: number, rate: number): number {
  if (rate === 0) return 0;
  const step = (nextIndex - previousIndex) * Math.sign(rate);
  if (step <= 0) return 0;
  const expected = Math.max(1, Math.round(Math.abs(rate)));
  return Math.max(0, step - expected);
}
