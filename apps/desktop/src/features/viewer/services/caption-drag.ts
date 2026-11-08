import type { Vec2 } from '@se/protocol';

import type { SnapGuides } from './gizmo-geometry';

/** Where a dragged caption's block may snap: the centre across, the thirds and centre down. */
const SNAP_X = [0.5];
const SNAP_Y = [1 / 3, 0.5, 2 / 3];

/**
 * The position (bottom centre, normalised) of a caption style dragged from `from` to `to`
 * (frame pixels), starting at `start`; snapped to the centre and thirds within `threshold` frame
 * pixels (0: off), kept inside the frame. Returns the guides to draw (frame pixels).
 */
export function draggedCaptionPosition(
  start: Vec2,
  from: Vec2,
  to: Vec2,
  frame: { readonly width: number; readonly height: number },
  threshold: number,
): { readonly position: Vec2; readonly guides: SnapGuides } {
  const clamp = (value: number) => Math.min(1, Math.max(0, value));
  let x = clamp(start.x + (to.x - from.x) / frame.width);
  let y = clamp(start.y + (to.y - from.y) / frame.height);
  const guides: { x: number[]; y: number[] } = { x: [], y: [] };
  const snapX = SNAP_X.find((target) => Math.abs(target - x) * frame.width <= threshold);
  if (snapX !== undefined) {
    x = snapX;
    guides.x.push(snapX * frame.width);
  }
  const snapY = SNAP_Y.find((target) => Math.abs(target - y) * frame.height <= threshold);
  if (snapY !== undefined) {
    y = snapY;
    guides.y.push(snapY * frame.height);
  }
  return { position: { x, y }, guides };
}
