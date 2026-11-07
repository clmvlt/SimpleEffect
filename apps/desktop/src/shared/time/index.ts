/**
 * Time in the renderer (ADR-0004): flicks, frame rates, timecodes and pixels, mirroring
 * `se_domain::time` and checked against `docs/test-vectors/*.json`; plus duration display.
 */
export {
  clampFlicks,
  FLICKS_PER_SECOND,
  flicksToFrames,
  flicksToSeconds,
  framesToFlicks,
  isFlicks,
  MAX_SAFE_FLICKS,
  secondsToFlicks,
  snapToFrame,
  ZERO_FLICKS,
} from './flicks';
export type { TimeErrorCode } from './flicks';
export { formatDuration } from './flicks-display';
export { formatFrameRate, frameDuration, frameRate, nominalFps } from './frame-rate';
export type { FrameRateErrorCode } from './frame-rate';
export { divideRounded } from './rounding';
export type { RoundingMode } from './rounding';
export { flicksToPixels, pixelsToFlicks } from './time-scale';
export type { TimeResult } from './time-result';
export { formatTimecode, parseTimecode } from './timecode';
export type { TimecodeErrorCode } from './timecode';
