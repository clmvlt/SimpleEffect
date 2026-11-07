import type { Asset, Format, ImageFit, ShapeGeometry } from '@se/protocol';

import type { Size } from './frame-graph';

/** The shorter side of a frame in which one design pixel is one real pixel (ADR-0015). */
export const DESIGN_REFERENCE_SIDE = 1080;

/** Real pixels per design pixel for a sequence format (`Format::design_scale` in Rust). */
export function designScale(format: Pick<Format, 'width' | 'height'>): number {
  return Math.min(format.width, format.height) / DESIGN_REFERENCE_SIDE;
}

/**
 * The upright picture size of a video or image asset in pixels: its coded size with the display
 * rotation applied (a 90° phone clip of 1920 × 1080 shows 1080 × 1920). `null` before the probe
 * reported it. Pixels are taken as square (the prober does not report an aspect ratio yet).
 */
export function assetDisplaySize(asset: Asset): Size | null {
  const stream = asset.mediaInfo?.videoStream;
  if (stream === undefined || stream === null || stream.width <= 0 || stream.height <= 0) {
    return null;
  }
  const quarterTurn = Math.abs(stream.rotation) % 180 === 90;
  return quarterTurn
    ? { width: stream.height, height: stream.width }
    : { width: stream.width, height: stream.height };
}

/**
 * The natural box of a picture of `display` size in a frame: scaled uniformly so it fits inside
 * (`fit`, letterboxed) or covers it (`fill`, cropped by the frame edges). Unknown sizes fill the
 * frame exactly.
 */
export function reframedBox(display: Size | null, frame: Size, mode: 'fit' | 'fill'): Size {
  if (display === null) return frame;
  const horizontal = frame.width / display.width;
  const vertical = frame.height / display.height;
  const scale = mode === 'fit' ? Math.min(horizontal, vertical) : Math.max(horizontal, vertical);
  return { width: display.width * scale, height: display.height * scale };
}

/**
 * Box of a shape in sequence pixels and its corner radius (design pixels × `scale`). A shape
 * covering the frame takes the frame's box (ADR-0015); a line is a capsule of its thickness.
 */
export function shapeBox(
  geometry: ShapeGeometry,
  scale: number,
  cover: { readonly frame: Size; readonly coverFrame: boolean } | null = null,
): { readonly box: Size; readonly cornerRadius: number } {
  if (geometry.kind === 'line') {
    const thickness = geometry.thickness * scale;
    return {
      box: { width: geometry.length * scale, height: thickness },
      cornerRadius: thickness / 2,
    };
  }
  const box =
    cover?.coverFrame === true
      ? cover.frame
      : { width: geometry.width * scale, height: geometry.height * scale };
  const cornerRadius = geometry.kind === 'rectangle' ? geometry.cornerRadius * scale : 0;
  return { box, cornerRadius };
}

/**
 * The natural box of an image of `display` size: fitted into or covering the frame, or one image
 * pixel per design pixel (`natural`, times `scale` = real pixels per design pixel).
 */
export function imageBox(display: Size | null, frame: Size, fit: ImageFit, scale: number): Size {
  if (fit !== 'natural') return reframedBox(display, frame, fit);
  if (display === null) return frame;
  return { width: display.width * scale, height: display.height * scale };
}
