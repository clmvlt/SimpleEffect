import type { ResolvedTransform, Size } from '@se/engine/evaluate';

/** An axis-aligned rectangle in frame pixels. */
export interface Bounds {
  readonly left: number;
  readonly top: number;
  readonly right: number;
  readonly bottom: number;
}

/**
 * The frame-pixel bounds of a box of `box` size placed by `transform` in a frame of `frame` size:
 * the anchor moved to the position, then scaled and rotated clockwise around it (the
 * compositor's placement, ADR-0015).
 */
export function layerBounds(transform: ResolvedTransform, box: Size, frame: Size): Bounds {
  const pivotX = transform.anchor.x * box.width;
  const pivotY = transform.anchor.y * box.height;
  const originX = transform.position.x * frame.width;
  const originY = transform.position.y * frame.height;
  const radians = (transform.rotation * Math.PI) / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);
  const corners: readonly (readonly [number, number])[] = [
    [0, 0],
    [box.width, 0],
    [0, box.height],
    [box.width, box.height],
  ];
  const xs: number[] = [];
  const ys: number[] = [];
  for (const [x, y] of corners) {
    const dx = (x - pivotX) * transform.scale.x;
    const dy = (y - pivotY) * transform.scale.y;
    xs.push(originX + dx * cos - dy * sin);
    ys.push(originY + dx * sin + dy * cos);
  }
  return {
    left: Math.min(...xs),
    top: Math.min(...ys),
    right: Math.max(...xs),
    bottom: Math.max(...ys),
  };
}

/** Whether `bounds` lie inside the frame, give or take `tolerance` pixels (rounding). */
export function insideFrame(bounds: Bounds, frame: Size, tolerance = 1): boolean {
  return (
    bounds.left >= -tolerance &&
    bounds.top >= -tolerance &&
    bounds.right <= frame.width + tolerance &&
    bounds.bottom <= frame.height + tolerance
  );
}
