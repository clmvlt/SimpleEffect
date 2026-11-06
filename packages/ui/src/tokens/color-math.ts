/**
 * Colour maths: parse `oklch()` and hex values, convert between OKLCH and sRGB, compute WCAG
 * contrast ratios. Used by the contrast test (AA invariant on text tokens), the Storybook Tokens
 * page and the `ColorField` picker (user colours). Out-of-gamut colours are clipped per channel, as
 * Chromium does when painting. It lives in `tokens/` because it is the one module allowed to spell
 * colour syntax.
 */

/** A colour in OKLCH (lightness 0–1, chroma, hue in degrees) with an alpha channel (0–1). */
export interface Oklch {
  readonly lightness: number;
  readonly chroma: number;
  readonly hue: number;
  readonly alpha: number;
}

/** A gamma-encoded sRGB colour, channels 0–1. */
export interface Srgb {
  readonly red: number;
  readonly green: number;
  readonly blue: number;
}

const OKLCH_PATTERN =
  /^oklch\(\s*([\d.]+)(%?)\s+([\d.]+)\s+([\d.]+)(?:deg)?\s*(?:\/\s*([\d.]+)(%?))?\s*\)$/;

/** Parses a CSS `oklch(L C H)` or `oklch(L C H / A)` value; `null` if it is not one. */
export function parseOklch(value: string): Oklch | null {
  const match = OKLCH_PATTERN.exec(value.trim());
  if (match === null) {
    return null;
  }
  const [, lightness = '0', lightnessPercent, chroma = '0', hue = '0', alpha, alphaPercent] = match;
  return {
    lightness: Number(lightness) / (lightnessPercent === '%' ? 100 : 1),
    chroma: Number(chroma),
    hue: Number(hue),
    alpha: alpha === undefined ? 1 : Number(alpha) / (alphaPercent === '%' ? 100 : 1),
  };
}

function clamp01(value: number): number {
  return Math.min(1, Math.max(0, value));
}

function encodeGamma(linear: number): number {
  const channel = clamp01(linear);
  return channel <= 0.0031308 ? 12.92 * channel : 1.055 * channel ** (1 / 2.4) - 0.055;
}

function decodeGamma(encoded: number): number {
  return encoded <= 0.04045 ? encoded / 12.92 : ((encoded + 0.055) / 1.055) ** 2.4;
}

/** OKLCH → linear sRGB channels, unclipped (values outside 0–1 are out of gamut). */
function oklchToLinearSrgb({ lightness, chroma, hue }: Oklch): [number, number, number] {
  const radians = (hue * Math.PI) / 180;
  const a = chroma * Math.cos(radians);
  const b = chroma * Math.sin(radians);
  const l = (lightness + 0.3963377774 * a + 0.2158037573 * b) ** 3;
  const m = (lightness - 0.1055613458 * a - 0.0638541728 * b) ** 3;
  const s = (lightness - 0.0894841775 * a - 1.291485548 * b) ** 3;
  return [
    4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
    -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
    -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s,
  ];
}

/** Converts OKLCH to gamma-encoded sRGB (alpha ignored), via OKLab (Björn Ottosson's matrices). */
export function oklchToSrgb(colour: Oklch): Srgb {
  const [red, green, blue] = oklchToLinearSrgb(colour);
  return { red: encodeGamma(red), green: encodeGamma(green), blue: encodeGamma(blue) };
}

/**
 * True when the colour is displayable in sRGB without clipping (2/1000 tolerance for rounding).
 * Tokens must be: a clipped colour is painted differently from what its OKLCH value says.
 */
export function isInSrgbGamut(colour: Oklch): boolean {
  return oklchToLinearSrgb(colour).every((channel) => channel > -0.002 && channel < 1.002);
}

/** Paints a translucent colour over an opaque background, as the browser does (in sRGB). */
export function compositeOver(foreground: Oklch, background: Srgb): Srgb {
  const top = oklchToSrgb(foreground);
  const mix = (over: number, under: number) =>
    over * foreground.alpha + under * (1 - foreground.alpha);
  return {
    red: mix(top.red, background.red),
    green: mix(top.green, background.green),
    blue: mix(top.blue, background.blue),
  };
}

/** WCAG 2 relative luminance of an sRGB colour. */
export function relativeLuminance({ red, green, blue }: Srgb): number {
  return 0.2126 * decodeGamma(red) + 0.7152 * decodeGamma(green) + 0.0722 * decodeGamma(blue);
}

/** WCAG 2 contrast ratio between two opaque colours (1 to 21; AA text needs 4.5). */
export function contrastRatio(first: Srgb, second: Srgb): number {
  const [lighter, darker] = [relativeLuminance(first), relativeLuminance(second)].sort(
    (x, y) => y - x,
  ) as [number, number];
  return (lighter + 0.05) / (darker + 0.05);
}

/**
 * Formats an sRGB colour as `#rrggbb` (channels clipped): for display next to a token, and as the
 * value of user colours picked in `ColorField`. Never used to style the UI itself.
 */
export function toHex({ red, green, blue }: Srgb): string {
  const channel = (value: number) =>
    Math.round(clamp01(value) * 255)
      .toString(16)
      .padStart(2, '0');
  return `#${channel(red)}${channel(green)}${channel(blue)}`;
}

/** Linear sRGB → OKLCH via OKLab (inverse of `oklchToLinearSrgb`). */
function linearSrgbToOklch(red: number, green: number, blue: number): Oklch {
  const l = Math.cbrt(0.4122214708 * red + 0.5363325363 * green + 0.0514459929 * blue);
  const m = Math.cbrt(0.2119034982 * red + 0.6806995451 * green + 0.1073969566 * blue);
  const s = Math.cbrt(0.0883024619 * red + 0.2817188376 * green + 0.6299787005 * blue);
  const lightness = 0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s;
  const a = 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s;
  const b = 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s;
  const chroma = Math.hypot(a, b);
  const hue = (Math.atan2(b, a) * 180) / Math.PI;
  return { lightness, chroma, hue: hue < 0 ? hue + 360 : hue, alpha: 1 };
}

/**
 * Converts gamma-encoded sRGB to OKLCH. Greys have no meaningful hue: their chroma is near 0 and
 * their hue is reported as 0, so pickers should keep their own hue while chroma is 0.
 */
export function srgbToOklch({ red, green, blue }: Srgb): Oklch {
  const colour = linearSrgbToOklch(decodeGamma(red), decodeGamma(green), decodeGamma(blue));
  return colour.chroma < 1e-4 ? { ...colour, chroma: 0, hue: 0 } : colour;
}

const HEX_PATTERN = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i;

/** Parses `#rgb` or `#rrggbb` (the `#` is optional, case ignored); `null` if it is neither. */
export function parseHex(text: string): Srgb | null {
  const digits = HEX_PATTERN.exec(text.trim())?.[1];
  if (digits === undefined) {
    return null;
  }
  const full =
    digits.length === 3 ? digits.replace(/[0-9a-f]/gi, (digit) => digit + digit) : digits;
  const channel = (index: number) => parseInt(full.slice(index, index + 2), 16) / 255;
  return { red: channel(0), green: channel(2), blue: channel(4) };
}
