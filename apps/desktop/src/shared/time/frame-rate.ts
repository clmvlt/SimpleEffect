import type { FrameRate } from '@se/protocol';

import { failure, success, type TimeResult } from './time-result';

/** Flicks in one second (repeated here so this module has no import cycle with `flicks.ts`). */
const FLICKS_PER_SECOND = 705_600_000;

/** Lowest and highest supported rates, in frames per second. */
const MIN_FPS = 1;
const MAX_FPS = 1000;

/** Why a frame rate is refused (Rust `TimeError` variants in camelCase). */
export type FrameRateErrorCode = 'zeroFrameRate' | 'frameRateOutOfRange' | 'inexactFrameRate';

function greatestCommonDivisor(a: number, b: number): number {
  let [x, y] = [a, b];
  while (y !== 0) [x, y] = [y, x % y];
  return x;
}

/**
 * The frame rate `numerator/denominator` reduced, or why it is refused: only rates of 1–1000 fps
 * whose frame lasts a whole number of flicks are representable (mirror of `FrameRate::new`).
 */
export function frameRate(
  numerator: number,
  denominator: number,
): TimeResult<FrameRate, FrameRateErrorCode> {
  if (numerator <= 0 || denominator <= 0) return failure('zeroFrameRate');
  const divisor = greatestCommonDivisor(numerator, denominator);
  const reduced = { numerator: numerator / divisor, denominator: denominator / divisor };
  if (
    reduced.numerator < MIN_FPS * reduced.denominator ||
    reduced.numerator > MAX_FPS * reduced.denominator
  ) {
    return failure('frameRateOutOfRange');
  }
  if ((FLICKS_PER_SECOND * reduced.denominator) % reduced.numerator !== 0) {
    return failure('inexactFrameRate');
  }
  return success(reduced);
}

/** Flicks in one frame at `rate` (exact for every valid rate, e.g. 23,543,520 at 29.97). */
export function frameDuration(rate: FrameRate): number {
  return (FLICKS_PER_SECOND * rate.denominator) / rate.numerator;
}

/** Frames counted per timecode second: the rate rounded up (30 for 29.97). */
export function nominalFps(rate: FrameRate): number {
  return Math.ceil(rate.numerator / rate.denominator);
}

/** The rate as shown to users: `25`, `29.97`, `23.976` (like Rust's `Display`). */
export function formatFrameRate(rate: FrameRate): string {
  if (rate.denominator === 1) return String(rate.numerator);
  return (rate.numerator / rate.denominator).toFixed(3).replace(/\.?0+$/u, '');
}
