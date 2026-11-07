import type { Crop, ResolvedTransform, Size } from '../evaluate/frame-graph';

/**
 * Where a layer's box lands in the sequence frame, in the terms of a 2D scene graph node: the box
 * is drawn in local coordinates `[0, width] × [0, height]`, `pivot` (the anchor in box pixels) is
 * moved to `(x, y)`, scaled, rotated clockwise (y points down) and faded by `alpha`.
 */
export interface LayerPlacement {
  readonly x: number;
  readonly y: number;
  readonly pivotX: number;
  readonly pivotY: number;
  readonly scaleX: number;
  readonly scaleY: number;
  /** Clockwise radians. */
  readonly rotation: number;
  readonly alpha: number;
}

/** The placement of a box of `box` size with `transform` in a frame of `frame` size. */
export function layerPlacement(
  transform: ResolvedTransform,
  box: Size,
  frame: Size,
): LayerPlacement {
  return {
    x: transform.position.x * frame.width,
    y: transform.position.y * frame.height,
    pivotX: transform.anchor.x * box.width,
    pivotY: transform.anchor.y * box.height,
    scaleX: transform.scale.x,
    scaleY: transform.scale.y,
    rotation: (transform.rotation * Math.PI) / 180,
    alpha: transform.opacity,
  };
}

/** A clockwise display rotation of decoded pixels, in degrees. */
export type PictureRotation = 0 | 90 | 180 | 270;

/**
 * How decoded pixels fill a layer's upright box: a sprite centred on the box (`x`, `y`), sized
 * `width` × `height` before its own rotation (the coded orientation), rotated by the display
 * rotation and mirrored horizontally when the stream says so.
 */
export interface PictureFit {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
  /** Clockwise radians. */
  readonly rotation: number;
  readonly mirrored: boolean;
}

/**
 * Stretches decoded pixels over `box` whatever their resolution: an original, its mezzanine and
 * its proxy all fill the same box, so switching variants never changes the framing.
 *
 * A scene graph scales before it rotates, so a mirror "after the rotation" is drawn as a mirror
 * before a rotation the other way (H·R(θ) = R(−θ)·H).
 */
export function pictureFit(box: Size, rotation: PictureRotation, mirrored: boolean): PictureFit {
  const quarterTurn = rotation === 90 || rotation === 270;
  const radians = (rotation * Math.PI) / 180;
  return {
    x: box.width / 2,
    y: box.height / 2,
    width: quarterTurn ? box.height : box.width,
    height: quarterTurn ? box.width : box.height,
    rotation: mirrored ? -radians : radians,
    mirrored,
  };
}

/** A normalised point of the upright picture as a point of the decoded (coded) pixels. */
function codedPoint(
  x: number,
  y: number,
  rotation: PictureRotation,
  mirrored: boolean,
): readonly [number, number] {
  // The upright picture is the coded one turned clockwise, then mirrored (see `pictureFit`):
  // undo the mirror, then the turn.
  const unmirrored = mirrored ? 1 - x : x;
  switch (rotation) {
    case 0:
      return [unmirrored, y];
    case 90:
      return [y, 1 - unmirrored];
    case 180:
      return [1 - unmirrored, 1 - y];
    case 270:
      return [1 - y, unmirrored];
  }
}

/**
 * The part of the decoded pixels showing `crop` (normalised in the upright picture) once they
 * are turned by `rotation` and mirrored: what a texture's frame must cover so the crop fills the
 * layer's box through {@link pictureFit}.
 */
export function codedCrop(crop: Crop, rotation: PictureRotation, mirrored: boolean): Crop {
  const [ax, ay] = codedPoint(crop.x, crop.y, rotation, mirrored);
  const [bx, by] = codedPoint(crop.x + crop.width, crop.y + crop.height, rotation, mirrored);
  return {
    x: Math.min(ax, bx),
    y: Math.min(ay, by),
    width: Math.abs(bx - ax),
    height: Math.abs(by - ay),
  };
}

/** Scale from sequence pixels to output pixels on each axis (the canvas may be any size). */
export function outputScale(frame: Size, output: Size): { readonly x: number; readonly y: number } {
  return { x: output.width / frame.width, y: output.height / frame.height };
}
