import type { Flicks, FrameRate } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../time/flicks';

/**
 * Without a known frame rate (variable-rate footage), media times become seconds a few ulps off
 * the container's timestamps: one microsecond lands inside the right frame (8.3 ms at 120 fps).
 */
const UNKNOWN_RATE_EPSILON = 1e-6;

/**
 * The time in seconds at which to look up the source frame shown at `sourceTime`.
 *
 * With the asset's frame rate, the frame index is computed exactly in flicks
 * (`⌊sourceTime / frame duration⌋`) and the lookup aims at the middle of that frame: containers
 * that round timestamps (Matroska and WebM store milliseconds, so 29.97 fps frames land up to
 * 0.5 ms off) still return the right frame, and a time one flick before a frame boundary still
 * shows the earlier frame.
 */
export function frameLookupTime(sourceTime: Flicks, frameRate: FrameRate | null): number {
  if (frameRate === null || frameRate.numerator <= 0 || frameRate.denominator <= 0) {
    return sourceTime / FLICKS_PER_SECOND + UNKNOWN_RATE_EPSILON;
  }
  const frameDuration = (FLICKS_PER_SECOND * frameRate.denominator) / frameRate.numerator;
  const index = Math.floor(sourceTime / frameDuration);
  return ((index + 0.5) * frameDuration) / FLICKS_PER_SECOND;
}
