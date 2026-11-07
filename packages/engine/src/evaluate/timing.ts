import type { Flicks, MediaClip, TimeRange } from '@se/protocol';

import { fadeCurve } from '../animation/fade-curve';
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
 * Level of a clip's fades at item-relative `localTime` (its picture's opacity factor and its
 * sound's gain factor): from 0 over `fadeIn` at the start and down to 0 over `fadeOut` at the end,
 * along each fade's shape (`fadeCurve`). Times beyond the item's edges (played during a
 * transition) take the level at the nearest edge. Overlapping fades take the lower level.
 */
export function fadeFactor(
  clip: Pick<MediaClip, 'fadeIn' | 'fadeOut' | 'fadeInShape' | 'fadeOutShape'>,
  duration: Flicks,
  localTime: number,
): number {
  const time = Math.min(duration, Math.max(0, localTime));
  let factor = 1;
  if (clip.fadeIn > 0 && time < clip.fadeIn) {
    factor = Math.min(factor, fadeCurve(clip.fadeInShape, time / clip.fadeIn));
  }
  const remaining = duration - time;
  if (clip.fadeOut > 0 && remaining < clip.fadeOut) {
    factor = Math.min(factor, fadeCurve(clip.fadeOutShape, remaining / clip.fadeOut));
  }
  return factor;
}
