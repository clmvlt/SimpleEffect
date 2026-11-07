import type { Reframe, Vec2 } from '@se/protocol';

import type { Crop, Size } from './frame-graph';
import { reframedBox } from './geometry';

/** A clip's reframe mode without its data (ADR-0026). */
export type ReframeMode = Reframe['kind'];

/** Where a clip's picture goes before its transform: its natural box and the part it shows. */
export interface ReframedPicture {
  /** The box the picture fills at scale 1, in sequence pixels. */
  readonly box: Size;
  /** The part of the upright picture stretched over the box; `null` for the whole picture. */
  readonly crop: Crop | null;
}

/** The centre of a picture: the focus when none is given. */
export const CENTRE_FOCUS: Vec2 = { x: 0.5, y: 0.5 };

function clamp(value: number, low: number, high: number): number {
  return Math.min(high, Math.max(low, value));
}

/**
 * The window of a picture of `display` size kept when it fills `frame` around `focus` (a point
 * normalised in the picture, kept at the frame's centre as far as the picture allows): the
 * frame's aspect, as large as the picture allows, never leaving it.
 */
export function fillCrop(display: Size, frame: Size, focus: Vec2): Crop {
  const cover = reframedBox(display, frame, 'fill');
  const width = Math.min(1, frame.width / cover.width);
  const height = Math.min(1, frame.height / cover.height);
  return {
    x: clamp(focus.x - width / 2, 0, 1 - width),
    y: clamp(focus.y - height / 2, 0, 1 - height),
    width,
    height,
  };
}

/**
 * The natural box and crop of a picture of `display` size in `frame` (domain `Reframe`,
 * ADR-0026): `fit` shows it whole inside the frame; `fill` and `tracked` crop it to the frame
 * around `focus` (the box is the frame); `custom` shows it whole at one picture pixel per design
 * pixel (`designScale` real pixels). Unknown sizes fill the frame exactly.
 */
export function reframedPicture(
  display: Size | null,
  frame: Size,
  mode: ReframeMode,
  focus: Vec2,
  designScale: number,
): ReframedPicture {
  if (display === null) return { box: frame, crop: null };
  switch (mode) {
    case 'fit':
      return { box: reframedBox(display, frame, 'fit'), crop: null };
    case 'custom':
      return {
        box: { width: display.width * designScale, height: display.height * designScale },
        crop: null,
      };
    case 'fill':
    case 'tracked':
      return { box: frame, crop: fillCrop(display, frame, focus) };
  }
}

/**
 * Sequence pixels per picture pixel of a picture of `display` size shown at scale 1 in `mode`;
 * `null` when the size is unknown. Switching between modes while keeping the picture's size
 * multiplies the scale by the ratio of these.
 */
export function reframeScale(
  display: Size | null,
  frame: Size,
  mode: ReframeMode,
  designScale: number,
): number | null {
  if (display === null || display.width <= 0 || display.height <= 0) return null;
  if (mode === 'custom') return designScale;
  const horizontal = frame.width / display.width;
  const vertical = frame.height / display.height;
  return mode === 'fit' ? Math.min(horizontal, vertical) : Math.max(horizontal, vertical);
}
