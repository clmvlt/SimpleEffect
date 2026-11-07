import type { ItemId, Vec2 } from '@se/protocol';

/** A width and a height in pixels of the sequence frame. */
export interface FrameSize {
  readonly width: number;
  readonly height: number;
}

/**
 * Where a layer sits (the domain's units, ADR-0015): `position` normalised in the frame,
 * relative `scale`, clockwise `rotation` in degrees, `anchor` normalised in the layer's box.
 */
export interface Placement {
  readonly position: Vec2;
  readonly scale: Vec2;
  readonly rotation: number;
  readonly anchor: Vec2;
}

/** A drawn layer: its item, placement and box in sequence pixels (the engine's layout). */
export interface PlacedLayer {
  readonly itemId: ItemId;
  readonly transform: Placement;
  readonly box: FrameSize;
}

/** The handles of the transform gizmo. */
export type ScaleHandle = 'n' | 's' | 'e' | 'w' | 'ne' | 'nw' | 'se' | 'sw';

/** Where each scale handle sits on the box, as fractions of its size. */
export const HANDLE_POSITIONS: Readonly<Record<ScaleHandle, Vec2>> = {
  nw: { x: 0, y: 0 },
  n: { x: 0.5, y: 0 },
  ne: { x: 1, y: 0 },
  e: { x: 1, y: 0.5 },
  se: { x: 1, y: 1 },
  s: { x: 0.5, y: 1 },
  sw: { x: 0, y: 1 },
  w: { x: 0, y: 0.5 },
};

const radians = (degrees: number) => (degrees * Math.PI) / 180;

/** The anchor of a placement in frame pixels (where the layer pivots). */
export function pivotOf(placement: Placement, frame: FrameSize): Vec2 {
  return { x: placement.position.x * frame.width, y: placement.position.y * frame.height };
}

/** A point of the layer's box (`local`, box pixels) in frame pixels: pivot + R·S·(local − anchor). */
export function toFrame(placement: Placement, box: FrameSize, frame: FrameSize, local: Vec2): Vec2 {
  const pivot = pivotOf(placement, frame);
  const dx = (local.x - placement.anchor.x * box.width) * placement.scale.x;
  const dy = (local.y - placement.anchor.y * box.height) * placement.scale.y;
  const angle = radians(placement.rotation);
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  return { x: pivot.x + dx * cos - dy * sin, y: pivot.y + dx * sin + dy * cos };
}

/** A frame point in the layer's box pixels (`null` when the layer is flattened to nothing). */
export function toLocal(
  placement: Placement,
  box: FrameSize,
  frame: FrameSize,
  point: Vec2,
): Vec2 | null {
  if (placement.scale.x === 0 || placement.scale.y === 0) return null;
  const pivot = pivotOf(placement, frame);
  const angle = radians(placement.rotation);
  const cos = Math.cos(angle);
  const sin = Math.sin(angle);
  const x = point.x - pivot.x;
  const y = point.y - pivot.y;
  return {
    x: (x * cos + y * sin) / placement.scale.x + placement.anchor.x * box.width,
    y: (-x * sin + y * cos) / placement.scale.y + placement.anchor.y * box.height,
  };
}

/** The four corners of a layer in frame pixels: top-left, top-right, bottom-right, bottom-left. */
export function cornersOf(layer: PlacedLayer, frame: FrameSize): readonly Vec2[] {
  const { width, height } = layer.box;
  return [
    { x: 0, y: 0 },
    { x: width, y: 0 },
    { x: width, y: height },
    { x: 0, y: height },
  ].map((local) => toFrame(layer.transform, layer.box, frame, local));
}

/** The topmost layer whose box contains `point` (frame pixels), or `null`. */
export function hitTest(
  layers: readonly PlacedLayer[],
  frame: FrameSize,
  point: Vec2,
): ItemId | null {
  for (let index = layers.length - 1; index >= 0; index -= 1) {
    const layer = layers[index];
    if (layer === undefined) continue;
    const local = toLocal(layer.transform, layer.box, frame, point);
    if (local === null) continue;
    if (local.x >= 0 && local.y >= 0 && local.x <= layer.box.width && local.y <= layer.box.height) {
      return layer.itemId;
    }
  }
  return null;
}

/** `start` moved by `delta` frame pixels. */
export function moved(start: Placement, frame: FrameSize, delta: Vec2): Placement {
  return {
    ...start,
    position: {
      x: start.position.x + delta.x / frame.width,
      y: start.position.y + delta.y / frame.height,
    },
  };
}

/**
 * `start` scaled around its anchor so the dragged `handle` follows `pointer` (frame pixels): side
 * handles change one axis, corners both; `proportional` (Shift) keeps the aspect ratio. An axis
 * whose handle lies on the anchor cannot scale and keeps its value.
 */
export function scaled(
  start: Placement,
  box: FrameSize,
  frame: FrameSize,
  handle: ScaleHandle,
  pointer: Vec2,
  proportional: boolean,
): Placement {
  const local = toLocal(start, box, frame, pointer);
  if (local === null) return start;
  const at = HANDLE_POSITIONS[handle];
  const factor = (axis: 'x' | 'y', size: number): number | null => {
    const span = (at[axis] - start.anchor[axis]) * size;
    if (Math.abs(span) < 1e-6) return null;
    return (local[axis] - start.anchor[axis] * size) / span;
  };
  const horizontal = handle.includes('e') || handle.includes('w') ? factor('x', box.width) : null;
  const vertical = handle.includes('n') || handle.includes('s') ? factor('y', box.height) : null;
  let fx = horizontal ?? 1;
  let fy = vertical ?? 1;
  if (proportional) {
    const uniform =
      horizontal === null ? fy : vertical === null ? fx : Math.abs(fx) > Math.abs(fy) ? fx : fy;
    fx = uniform;
    fy = uniform;
  }
  return { ...start, scale: { x: start.scale.x * fx, y: start.scale.y * fy } };
}

/**
 * `start` rotated by the angle the pointer turned around the anchor, from `from` to `to` (frame
 * pixels); `snap` (Shift) rounds to 15°.
 */
export function rotated(
  start: Placement,
  frame: FrameSize,
  from: Vec2,
  to: Vec2,
  snap: boolean,
): Placement {
  const pivot = pivotOf(start, frame);
  const angle = (point: Vec2) => (Math.atan2(point.y - pivot.y, point.x - pivot.x) * 180) / Math.PI;
  let rotation = start.rotation + angle(to) - angle(from);
  if (snap) rotation = Math.round(rotation / 15) * 15;
  return { ...start, rotation: Math.round(rotation * 100) / 100 };
}

/**
 * `start` with its anchor moved to `point` (frame pixels) without moving the layer: the anchor
 * becomes that point of the box and the position follows it.
 */
export function reanchored(
  start: Placement,
  box: FrameSize,
  frame: FrameSize,
  point: Vec2,
): Placement {
  const local = toLocal(start, box, frame, point);
  if (local === null || box.width === 0 || box.height === 0) return start;
  return {
    ...start,
    anchor: { x: local.x / box.width, y: local.y / box.height },
    position: { x: point.x / frame.width, y: point.y / frame.height },
  };
}

/** Snap lines shown while moving: frame pixels on each axis. */
export interface SnapGuides {
  readonly x: readonly number[];
  readonly y: readonly number[];
}

/**
 * Snaps a moved layer to the frame's centre and edges: the edges and centre of the layer's
 * bounds (rotation included) closer than `threshold` frame pixels to a frame line are pulled onto
 * it, one axis at a time. Returns the adjusted placement and the lines it snapped to.
 */
export function snapToFrame(
  placement: Placement,
  box: FrameSize,
  frame: FrameSize,
  threshold: number,
): { readonly placement: Placement; readonly guides: SnapGuides } {
  const corners = cornersOf({ itemId: '', transform: placement, box }, frame);
  const xs = corners.map((corner) => corner.x);
  const ys = corners.map((corner) => corner.y);
  const bounds = (values: number[]) => {
    const low = Math.min(...values);
    const high = Math.max(...values);
    return [low, (low + high) / 2, high];
  };
  const best = (edges: number[], lines: number[]) => {
    let found: { shift: number; line: number } | null = null;
    for (const edge of edges) {
      for (const line of lines) {
        const shift = line - edge;
        if (
          Math.abs(shift) <= threshold &&
          (found === null || Math.abs(shift) < Math.abs(found.shift))
        ) {
          found = { shift, line };
        }
      }
    }
    return found;
  };
  const snapX = best(bounds(xs), [0, frame.width / 2, frame.width]);
  const snapY = best(bounds(ys), [0, frame.height / 2, frame.height]);
  return {
    placement: moved(placement, frame, { x: snapX?.shift ?? 0, y: snapY?.shift ?? 0 }),
    guides: { x: snapX === null ? [] : [snapX.line], y: snapY === null ? [] : [snapY.line] },
  };
}
