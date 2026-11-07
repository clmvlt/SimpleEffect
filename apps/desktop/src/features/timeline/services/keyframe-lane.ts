import type {
  AnimatableProperty,
  Easing,
  Flicks,
  ItemAnimation,
  ItemEntity,
  ItemKind,
} from '@se/protocol';

import {
  ANIMATABLE_PROPERTIES,
  animationOf,
  movedKeyframes,
  propertyOf,
  withEasing,
  withoutKeyframes,
} from '../../../shared/animation';
import type { KeyframeKey } from '../store/keyframe-store';

/** Height of one property row of a keyframe lane, in pixels. */
export const KEYFRAME_ROW_HEIGHT = 16;

/** The properties of `kind` that have keyframes, in inspector order (`volumeDb` has its line). */
export function animatedProperties(kind: ItemKind): AnimatableProperty[] {
  return ANIMATABLE_PROPERTIES.filter(
    (property) => property !== 'volumeDb' && propertyOf(kind, property)?.kind === 'keyframed',
  );
}

/** Height of the keyframe lane of an item of `kind` (one row per animated property). */
export function keyframeLaneHeight(kind: ItemKind): number {
  return animatedProperties(kind).length * KEYFRAME_ROW_HEIGHT;
}

/** Keyframe times of `property` on `kind` (item-relative). */
export function keyframeTimes(kind: ItemKind, property: AnimatableProperty): Flicks[] {
  const value = propertyOf(kind, property);
  return value?.kind === 'keyframed' ? value.keyframes.map((keyframe) => keyframe.time) : [];
}

/** The keys of `keys` on `property`, as times. */
function timesOf(keys: readonly KeyframeKey[], property: AnimatableProperty): Flicks[] {
  return keys.filter((key) => key.property === property).map((key) => key.time);
}

/** The properties `keys` touch, each once. */
function propertiesOf(keys: readonly KeyframeKey[]): AnimatableProperty[] {
  return [...new Set(keys.map((key) => key.property))];
}

/**
 * The new values of the properties whose keyframes `keys` are moved by `delta` (kept inside the
 * item); `null` when a moved keyframe would land on another one.
 */
export function movedKeyframeAnimations(
  item: ItemEntity,
  keys: readonly KeyframeKey[],
  delta: number,
): ItemAnimation[] | null {
  const changes: ItemAnimation[] = [];
  for (const property of propertiesOf(keys)) {
    const value = propertyOf(item.kind, property);
    if (value === null) continue;
    const moved = movedKeyframes(
      value,
      timesOf(keys, property),
      delta,
      item.timelineRange.duration,
    );
    if (moved === null) return null;
    changes.push({ itemId: item.id, animation: animationOf(property, moved) });
  }
  return changes;
}

/** The new values of the properties whose keyframes `keys` are deleted. */
export function deletedKeyframeAnimations(
  item: ItemEntity,
  keys: readonly KeyframeKey[],
): ItemAnimation[] {
  return propertiesOf(keys).flatMap((property) => {
    const value = propertyOf(item.kind, property);
    if (value === null) return [];
    const rest = withoutKeyframes(value, timesOf(keys, property));
    return [{ itemId: item.id, animation: animationOf(property, rest) }];
  });
}

/** The new value of `key`'s property with `easing` leaving that keyframe. */
export function easedKeyframeAnimation(
  item: ItemEntity,
  key: KeyframeKey,
  easing: Easing,
): ItemAnimation | null {
  const value = propertyOf(item.kind, key.property);
  if (value === null) return null;
  return {
    itemId: item.id,
    animation: animationOf(key.property, withEasing(value, key.time, easing)),
  };
}

/** The easing leaving `key` (linear when the keyframe is gone). */
export function easingOf(item: ItemEntity, key: KeyframeKey): Easing {
  const value = propertyOf(item.kind, key.property);
  const keyframe =
    value?.kind === 'keyframed'
      ? value.keyframes.find((candidate) => candidate.time === key.time)
      : undefined;
  return keyframe?.easing ?? { kind: 'linear' };
}
