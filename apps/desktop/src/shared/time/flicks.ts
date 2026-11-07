import type { Flicks, FrameRate } from '@se/protocol';

import { frameDuration } from './frame-rate';
import { divideRounded, type RoundingMode } from './rounding';
import { failure, success, type TimeResult } from './time-result';

/**
 * Flicks in TypeScript (ADR-0004, mirror of `se_domain::Flicks`): a branded integer `number`,
 * 705,600,000 per second, within ±(2^53 − 1). Floats appear only at boundaries (display, pixels).
 */

/** Flicks in one second. */
export const FLICKS_PER_SECOND = 705_600_000;

/** Largest magnitude a flicks value may have: `Number.MAX_SAFE_INTEGER` (about 147 days). */
export const MAX_SAFE_FLICKS = Number.MAX_SAFE_INTEGER;

/** Zero flicks. */
export const ZERO_FLICKS = 0 as Flicks;

/** Why a time cannot be represented (Rust `TimeError` variants in camelCase). */
export type TimeErrorCode = 'outOfRange' | 'notFinite';

/** Whether `value` is an integer within the flicks range. */
export function isFlicks(value: number): value is Flicks {
  return Number.isSafeInteger(value);
}

/**
 * `value` as flicks, clamped into the range and rounded to an integer: for values computed in the
 * UI (pixels, drags) that only need to stay valid.
 */
export function clampFlicks(value: number): Flicks {
  if (Number.isNaN(value)) return ZERO_FLICKS;
  const clamped = Math.min(MAX_SAFE_FLICKS, Math.max(-MAX_SAFE_FLICKS, value));
  return roundHalfAwayFromZero(clamped) as Flicks;
}

/** Seconds of a flicks value (display and pixels only, never for document positions). */
export function flicksToSeconds(time: Flicks): number {
  return time / FLICKS_PER_SECOND;
}

/** The nearest flick to `seconds` (halves away from zero), or why it cannot be represented. */
export function secondsToFlicks(seconds: number): TimeResult<Flicks, TimeErrorCode> {
  if (!Number.isFinite(seconds)) return failure('notFinite');
  const rounded = roundHalfAwayFromZero(seconds * FLICKS_PER_SECOND);
  return Math.abs(rounded) > MAX_SAFE_FLICKS ? failure('outOfRange') : success(rounded as Flicks);
}

/** The start of frame `frames` at `rate`, or `outOfRange`. */
export function framesToFlicks(frames: number, rate: FrameRate): TimeResult<Flicks, 'outOfRange'> {
  const product = frames * frameDuration(rate);
  return Number.isSafeInteger(product) ? success(product as Flicks) : failure('outOfRange');
}

/** The frame number of `time` at `rate`, rounded as `rounding` says. */
export function flicksToFrames(time: Flicks, rate: FrameRate, rounding: RoundingMode): number {
  return divideRounded(time, frameDuration(rate), rounding);
}

/** `time` moved to the start of the frame it falls in (or the nearest frame start). */
export function snapToFrame(time: Flicks, rate: FrameRate, rounding: RoundingMode): Flicks {
  return clampFlicks(flicksToFrames(time, rate, rounding) * frameDuration(rate));
}

/** `Math.round` with halves away from zero, like Rust's `f64::round`. */
function roundHalfAwayFromZero(value: number): number {
  return Math.sign(value) * Math.round(Math.abs(value));
}
