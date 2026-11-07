import type { Vec2 } from '@se/protocol';
import type { Crop } from '@se/engine/evaluate';

import { toFrame, toLocal, type FrameSize, type PlacedLayer } from './gizmo-geometry';

/** A drawn clip that shows part of its picture (a fill mode, ADR-0026). */
export interface CroppedLayer extends PlacedLayer {
  readonly crop: Crop;
}

/** A rectangle in a layer's box pixels. */
export interface BoxRect {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/**
 * The whole picture of a cropped layer in its box pixels: the box shows `crop`, so the picture
 * extends beyond it by the parts cropped away.
 */
export function pictureRect({ box, crop }: CroppedLayer): BoxRect {
  const width = box.width / crop.width;
  const height = box.height / crop.height;
  return { x: -crop.x * width, y: -crop.y * height, width, height };
}

/** The corners of the whole picture in frame pixels (top-left, top-right, bottom-right, bottom-left). */
export function pictureCorners(layer: CroppedLayer, frame: FrameSize): readonly Vec2[] {
  const { x, y, width, height } = pictureRect(layer);
  return [
    { x, y },
    { x: x + width, y },
    { x: x + width, y: y + height },
    { x, y: y + height },
  ].map((local) => toFrame(layer.transform, layer.box, frame, local));
}

/** The focus the layer shows: the centre of its crop (a stored focus may lie beyond the reach). */
export function shownFocus(crop: Crop): Vec2 {
  return { x: crop.x + crop.width / 2, y: crop.y + crop.height / 2 };
}

function clampAxis(value: number, window: number): number {
  const half = window / 2;
  return Math.min(1 - half, Math.max(half, value));
}

/**
 * The focus after dragging the picture from `from` to `to` (frame pixels): the picture follows
 * the pointer, so the kept part moves the other way; the focus stays where the crop can reach.
 * `null` when the layer is flattened to nothing.
 */
export function pannedFocus(
  layer: CroppedLayer,
  frame: FrameSize,
  from: Vec2,
  to: Vec2,
): Vec2 | null {
  const start = toLocal(layer.transform, layer.box, frame, from);
  const end = toLocal(layer.transform, layer.box, frame, to);
  if (start === null || end === null) return null;
  const picture = pictureRect(layer);
  const focus = shownFocus(layer.crop);
  return {
    x: clampAxis(focus.x - (end.x - start.x) / picture.width, layer.crop.width),
    y: clampAxis(focus.y - (end.y - start.y) / picture.height, layer.crop.height),
  };
}

/**
 * The CSS transform drawing a box-pixel rectangle of `layer` at its place in display pixels
 * (`scale` CSS pixels per frame pixel): `matrix(a, b, c, d, e, f)` for an element sized
 * `rect.width` × `rect.height` CSS pixels with `transform-origin: 0 0`.
 */
export function boxRectTransform(
  layer: PlacedLayer,
  frame: FrameSize,
  rect: BoxRect,
  scale: number,
): string {
  const origin = toFrame(layer.transform, layer.box, frame, { x: rect.x, y: rect.y });
  const radians = (layer.transform.rotation * Math.PI) / 180;
  const cos = Math.cos(radians);
  const sin = Math.sin(radians);
  const { x: scaleX, y: scaleY } = layer.transform.scale;
  const values = [
    cos * scaleX * scale,
    sin * scaleX * scale,
    -sin * scaleY * scale,
    cos * scaleY * scale,
    origin.x * scale,
    origin.y * scale,
  ];
  return `matrix(${values.map((value) => String(value)).join(', ')})`;
}
