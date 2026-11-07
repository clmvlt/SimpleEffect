import type { Flicks, Transform2D, Vec2 } from '@se/protocol';

import { interpolateNumber, interpolateVec2, valueAt } from '../animation/interpolate';
import type { ResolvedTransform } from './frame-graph';

const CENTRE: Vec2 = { x: 0.5, y: 0.5 };
const UNIT: Vec2 = { x: 1, y: 1 };

/** Every property of `transform` at item-relative `localTime`, opacity times `opacityFactor`. */
export function resolveTransform(
  transform: Transform2D,
  localTime: Flicks,
  opacityFactor = 1,
): ResolvedTransform {
  const opacity = valueAt(transform.opacity, localTime, interpolateNumber, 1);
  return {
    position: valueAt(transform.position, localTime, interpolateVec2, CENTRE),
    scale: valueAt(transform.scale, localTime, interpolateVec2, UNIT),
    rotation: valueAt(transform.rotation, localTime, interpolateNumber, 0),
    anchor: valueAt(transform.anchor, localTime, interpolateVec2, CENTRE),
    opacity: Math.min(1, Math.max(0, opacity * opacityFactor)),
  };
}

/**
 * Whether a resolved transform draws nothing (fully transparent or flattened to a line): such
 * layers are left out of the frame graph, so their media is not decoded.
 */
export function isInvisible(transform: ResolvedTransform): boolean {
  return transform.opacity <= 0 || transform.scale.x === 0 || transform.scale.y === 0;
}
