import type { Animatable, Easing, Flicks, Keyframe } from '@se/protocol';
import { valueAt, type Interpolator } from '@se/engine/animation';

/** Where the playhead stands relative to a property's keyframes (the `KeyframeButton` states). */
export type KeyframeState = 'none' | 'betweenKeyframes' | 'onKeyframe';

/** Easing given to new keyframes. */
export const DEFAULT_EASING: Easing = { kind: 'linear' };

/**
 * Index of the keyframe within `tolerance` of `time` (item-relative), or `-1`. The tolerance is
 * half a frame in the UI: the playhead sits on frame starts, keyframes may not.
 */
export function keyframeIndexAt<T>(
  property: Animatable<T>,
  time: Flicks,
  tolerance: number,
): number {
  if (property.kind === 'static') return -1;
  return property.keyframes.findIndex((keyframe) => Math.abs(keyframe.time - time) <= tolerance);
}

/** The keyframe button's state for `property` at `time`. */
export function keyframeState<T>(
  property: Animatable<T>,
  time: Flicks,
  tolerance: number,
): KeyframeState {
  if (property.kind === 'static') return 'none';
  return keyframeIndexAt(property, time, tolerance) >= 0 ? 'onKeyframe' : 'betweenKeyframes';
}

/** The value of `property` at `time` (the engine's interpolation, so it matches the picture). */
export function valueAtTime<T>(
  property: Animatable<T>,
  time: Flicks,
  interpolate: Interpolator<T>,
): T {
  const fallback = property.kind === 'static' ? property.value : property.keyframes[0]?.value;
  // A keyframed property always has a keyframe (domain invariant).
  return valueAt(property, time, interpolate, fallback as T);
}

function sorted<T>(keyframes: Keyframe<T>[]): Animatable<T> {
  return { kind: 'keyframed', keyframes: keyframes.sort((left, right) => left.time - right.time) };
}

/**
 * `property` holding `value` at `time`: a static property takes the value; a keyframed one gets
 * its keyframe at `time` changed, or a new keyframe there (auto-keyframing, as in motion tools).
 */
export function withValueAt<T>(
  property: Animatable<T>,
  time: Flicks,
  value: T,
  tolerance: number,
): Animatable<T> {
  if (property.kind === 'static') return { kind: 'static', value };
  const index = keyframeIndexAt(property, time, tolerance);
  const keyframes = [...property.keyframes];
  const existing = keyframes[index];
  if (existing !== undefined) keyframes[index] = { ...existing, value };
  else keyframes.push({ time, value, easing: easingBefore(property.keyframes, time) });
  return sorted(keyframes);
}

/** A new keyframe inherits the easing of the segment it splits (linear past the ends). */
function easingBefore<T>(keyframes: readonly Keyframe<T>[], time: Flicks): Easing {
  const previous = keyframes.filter((keyframe) => keyframe.time < time).at(-1);
  const next = keyframes.find((keyframe) => keyframe.time > time);
  return previous !== undefined && next !== undefined ? previous.easing : DEFAULT_EASING;
}

/**
 * The keyframe button's action: removes the keyframe at `time` (the property becomes static with
 * that value when it was the last one), or adds one there holding the current value.
 */
export function toggledKeyframe<T>(
  property: Animatable<T>,
  time: Flicks,
  tolerance: number,
  interpolate: Interpolator<T>,
): Animatable<T> {
  const index = keyframeIndexAt(property, time, tolerance);
  if (property.kind === 'keyframed' && index >= 0) {
    const removed = property.keyframes[index];
    const rest = property.keyframes.filter((_, at) => at !== index);
    if (rest.length > 0) return { kind: 'keyframed', keyframes: rest };
    return removed === undefined ? property : { kind: 'static', value: removed.value };
  }
  const value = valueAtTime(property, time, interpolate);
  if (property.kind === 'static') {
    return { kind: 'keyframed', keyframes: [{ time, value, easing: DEFAULT_EASING }] };
  }
  return withValueAt(property, time, value, -1);
}

/** Time of the nearest keyframe before (`-1`) or after (`1`) `time`, beyond `tolerance`. */
export function neighbourKeyframe<T>(
  property: Animatable<T>,
  time: Flicks,
  direction: -1 | 1,
  tolerance: number,
): Flicks | null {
  if (property.kind === 'static') return null;
  const times = property.keyframes.map((keyframe) => keyframe.time);
  const found =
    direction < 0
      ? times.filter((at) => at < time - tolerance).at(-1)
      : times.find((at) => at > time + tolerance);
  return found ?? null;
}

/**
 * `property` with its keyframes at `times` moved by `delta`, the shift clamped so they stay in
 * `[0, duration]`; `null` when a moved keyframe would land on another one.
 */
export function movedKeyframes<T>(
  property: Animatable<T>,
  times: readonly Flicks[],
  delta: number,
  duration: Flicks,
): Animatable<T> | null {
  if (property.kind === 'static') return property;
  const moving = property.keyframes.filter((keyframe) => times.includes(keyframe.time));
  if (moving.length === 0) return property;
  const earliest = Math.min(...moving.map((keyframe) => keyframe.time));
  const latest = Math.max(...moving.map((keyframe) => keyframe.time));
  const shift = Math.min(Math.max(delta, -earliest), duration - latest);
  const moved = property.keyframes.map((keyframe) =>
    times.includes(keyframe.time)
      ? { ...keyframe, time: (keyframe.time + Math.round(shift)) as Flicks }
      : keyframe,
  );
  const unique = new Set(moved.map((keyframe) => keyframe.time));
  return unique.size === moved.length ? sorted(moved) : null;
}

/** `property` without its keyframes at `times` (static with the first one's value if none is left). */
export function withoutKeyframes<T>(
  property: Animatable<T>,
  times: readonly Flicks[],
): Animatable<T> {
  if (property.kind === 'static') return property;
  const rest = property.keyframes.filter((keyframe) => !times.includes(keyframe.time));
  if (rest.length > 0) return { kind: 'keyframed', keyframes: rest };
  const first = property.keyframes[0];
  return first === undefined ? property : { kind: 'static', value: first.value };
}

/** `property` with the keyframe at `time` easing to the next one with `easing`. */
export function withEasing<T>(
  property: Animatable<T>,
  time: Flicks,
  easing: Easing,
): Animatable<T> {
  if (property.kind === 'static') return property;
  return {
    kind: 'keyframed',
    keyframes: property.keyframes.map((keyframe) =>
      keyframe.time === time ? { ...keyframe, easing } : keyframe,
    ),
  };
}
