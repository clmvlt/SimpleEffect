import type { Animatable, Color, Flicks, Keyframe, Vec2 } from '@se/protocol';

import { BLACK, formatColor, mixColors, parseColor } from './color';
import { easedProgress } from './easing';

/** Blends two values of a property: `progress` 0 gives `from`, 1 gives `to` (may overshoot). */
export type Interpolator<T> = (from: T, to: T, progress: number) => T;

/** Numbers blend linearly. */
export const interpolateNumber: Interpolator<number> = (from, to, progress) =>
  from + (to - from) * progress;

/** Vectors (positions, anchors, scales) blend component by component. */
export const interpolateVec2: Interpolator<Vec2> = (from, to, progress) => ({
  x: interpolateNumber(from.x, to.x, progress),
  y: interpolateNumber(from.y, to.y, progress),
});

/** Colours blend in OKLab with premultiplied alpha (see `mixColors`). */
export const interpolateColor: Interpolator<Color> = (from, to, progress) =>
  formatColor(mixColors(parseColor(from) ?? BLACK, parseColor(to) ?? BLACK, progress));

/**
 * Index of the last keyframe at or before `time` (`-1` when `time` is before the first one), by
 * binary search: keyframes are sorted by strictly increasing time (domain invariant).
 */
function lastKeyframeAtOrBefore<T>(keyframes: readonly Keyframe<T>[], time: Flicks): number {
  let low = 0;
  let high = keyframes.length - 1;
  let found = -1;
  while (low <= high) {
    const middle = (low + high) >> 1;
    const keyframe = keyframes[middle];
    if (keyframe !== undefined && keyframe.time <= time) {
      found = middle;
      low = middle + 1;
    } else {
      high = middle - 1;
    }
  }
  return found;
}

/**
 * Value of a keyframe list at `time` (item-relative flicks): the first value before the first
 * keyframe, the last after the last one, otherwise the blend of the surrounding pair shaped by
 * the earlier keyframe's easing. A `hold` keeps its value until the next keyframe's exact time.
 */
export function keyframesValueAt<T>(
  keyframes: readonly Keyframe<T>[],
  time: Flicks,
  interpolate: Interpolator<T>,
): T | undefined {
  const index = lastKeyframeAtOrBefore(keyframes, time);
  const current = keyframes[Math.max(index, 0)];
  const next = keyframes[index + 1];
  if (current === undefined) return undefined;
  if (index < 0 || next === undefined) return current.value;
  const progress = (time - current.time) / (next.time - current.time);
  return interpolate(current.value, next.value, easedProgress(current.easing, progress));
}

/**
 * Value of an animatable property at `time` (item-relative flicks). A keyframed property always
 * has a keyframe (domain invariant); `fallback` only guards against malformed input.
 */
export function valueAt<T>(
  property: Animatable<T>,
  time: Flicks,
  interpolate: Interpolator<T>,
  fallback: T,
): T {
  if (property.kind === 'static') return property.value;
  return keyframesValueAt(property.keyframes, time, interpolate) ?? fallback;
}
