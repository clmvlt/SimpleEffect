import type { Color } from '@se/protocol';

/** A colour with straight (non-premultiplied) alpha, every channel in `[0, 1]` (sRGB encoded). */
export interface Rgba {
  readonly r: number;
  readonly g: number;
  readonly b: number;
  readonly a: number;
}

/** A colour in OKLab (perceptual lightness `l`, opponent axes `a` and `b`) with alpha. */
export interface OklabColor {
  readonly l: number;
  readonly a: number;
  readonly b: number;
  readonly alpha: number;
}

const HEX_COLOR = /^#([0-9a-f]{6})([0-9a-f]{2})?$/i;

/** Opaque black, used when a document colour cannot be read (the domain validates them). */
export const BLACK: Rgba = { r: 0, g: 0, b: 0, a: 1 };

/** Reads a document colour (`#rrggbb` or `#rrggbbaa`, any case); `null` if malformed. */
export function parseColor(color: Color): Rgba | null {
  const match = HEX_COLOR.exec(color);
  if (match === null) return null;
  const digits = (match[1] ?? '') + (match[2] ?? 'ff');
  const channel = (index: number) => Number.parseInt(digits.slice(index, index + 2), 16) / 255;
  return { r: channel(0), g: channel(2), b: channel(4), a: channel(6) };
}

/** Writes a colour the way the domain serializes it (`#rrggbb`, or `#rrggbbaa` when translucent). */
export function formatColor(color: Rgba): Color {
  const hex = (value: number) =>
    Math.round(Math.min(1, Math.max(0, value)) * 255)
      .toString(16)
      .padStart(2, '0');
  const alpha = Math.round(Math.min(1, Math.max(0, color.a)) * 255);
  return `#${hex(color.r)}${hex(color.g)}${hex(color.b)}${alpha === 255 ? '' : hex(color.a)}`;
}

function toLinear(channel: number): number {
  return channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4;
}

function toGamma(channel: number): number {
  return channel <= 0.0031308 ? channel * 12.92 : 1.055 * channel ** (1 / 2.4) - 0.055;
}

/** sRGB → OKLab (Björn Ottosson's matrices, as in CSS Color 4). */
export function rgbaToOklab(color: Rgba): OklabColor {
  const r = toLinear(color.r);
  const g = toLinear(color.g);
  const b = toLinear(color.b);
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b);
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b);
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b);
  return {
    l: 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s,
    a: 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s,
    b: 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s,
    alpha: color.a,
  };
}

/** OKLab → sRGB, clamped into the sRGB gamut. */
export function oklabToRgba(color: OklabColor): Rgba {
  const l = (color.l + 0.3963377774 * color.a + 0.2158037573 * color.b) ** 3;
  const m = (color.l - 0.1055613458 * color.a - 0.0638541728 * color.b) ** 3;
  const s = (color.l - 0.0894841775 * color.a - 1.291485548 * color.b) ** 3;
  const clamp = (value: number) => Math.min(1, Math.max(0, toGamma(value)));
  return {
    r: clamp(4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s),
    g: clamp(-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s),
    b: clamp(-0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s),
    a: Math.min(1, Math.max(0, color.alpha)),
  };
}

/**
 * The colour `progress` of the way from `from` to `to`, interpolated in OKLab with premultiplied
 * alpha (as CSS Color 4 does): perceptually even, and fading from a transparent colour does not
 * drag the hue of its invisible RGB.
 */
export function mixColors(from: Rgba, to: Rgba, progress: number): Rgba {
  const start = rgbaToOklab(from);
  const end = rgbaToOklab(to);
  const alpha = start.alpha + (end.alpha - start.alpha) * progress;
  if (alpha <= 0) {
    return { ...oklabToRgba({ ...start, alpha: 0 }), a: 0 };
  }
  const premultiplied = (channel: 'l' | 'a' | 'b') =>
    (start[channel] * start.alpha +
      (end[channel] * end.alpha - start[channel] * start.alpha) * progress) /
    alpha;
  return oklabToRgba({
    l: premultiplied('l'),
    a: premultiplied('a'),
    b: premultiplied('b'),
    alpha,
  });
}
