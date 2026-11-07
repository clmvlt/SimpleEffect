import type { Flicks, MediaClip, TimeRange } from '@se/protocol';

import { roundToFlicks } from '../time/flicks';

/**
 * Time in the asset shown by a clip at sequence `time`:
 * `sourceStart + (time − start) × speed`, rounded to the nearest flick.
 */
export function clipSourceTime(
  clip: Pick<MediaClip, 'sourceStart' | 'speed'>,
  range: TimeRange,
  time: Flicks,
): Flicks {
  return roundToFlicks(clip.sourceStart + (time - range.start) * clip.speed);
}

/**
 * Picture opacity factor of a clip's fades at item-relative `localTime`: a linear ramp from 0 over
 * `fadeIn` at the start and down to 0 over `fadeOut` at the end ("from/to black" over the frame's
 * black background; translucent over lower tracks).
 */
export function fadeFactor(
  clip: Pick<MediaClip, 'fadeIn' | 'fadeOut'>,
  duration: Flicks,
  localTime: Flicks,
): number {
  let factor = 1;
  if (clip.fadeIn > 0 && localTime < clip.fadeIn) {
    factor = Math.min(factor, localTime / clip.fadeIn);
  }
  const remaining = duration - localTime;
  if (clip.fadeOut > 0 && remaining < clip.fadeOut) {
    factor = Math.min(factor, remaining / clip.fadeOut);
  }
  return Math.min(1, Math.max(0, factor));
}
