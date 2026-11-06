import { oklchToSrgb, parseHex, srgbToOklch, toHex } from '../../tokens/color-math';

/** The colour being edited, in OKLCH (perceptual: equal steps look equally different). */
export interface PickerColor {
  /** 0–1. */
  readonly lightness: number;
  /** 0–`MAX_CHROMA`. */
  readonly chroma: number;
  /** 0–360 degrees. */
  readonly hue: number;
}

/** One OKLCH channel of the picker. */
export type ColorChannel = keyof PickerColor;

/** Highest chroma offered: beyond it almost nothing is displayable in sRGB. */
export const MAX_CHROMA = 0.37;

/** Range and keyboard step of each channel. */
export const CHANNEL_RANGES: Readonly<
  Record<ColorChannel, { min: number; max: number; step: number }>
> = {
  lightness: { min: 0, max: 1, step: 0.01 },
  chroma: { min: 0, max: MAX_CHROMA, step: 0.005 },
  hue: { min: 0, max: 360, step: 1 },
};

/** `#rrggbb` of a picker colour (out-of-gamut channels are clipped, as browsers paint them). */
export function hexFromColor(color: PickerColor): string {
  return toHex(oklchToSrgb({ ...color, alpha: 1 }));
}

/**
 * Picker colour of a hex value, or `null` if the text is not a hex colour. Greys have no hue:
 * `fallbackHue` (the hue currently shown) is kept so the hue slider does not jump to red.
 */
export function colorFromHex(hex: string, fallbackHue = 0): PickerColor | null {
  const srgb = parseHex(hex);
  if (srgb === null) {
    return null;
  }
  const { lightness, chroma, hue } = srgbToOklch(srgb);
  return { lightness, chroma, hue: chroma === 0 ? fallbackHue : hue };
}

/** Normalises a hex text to `#rrggbb`, or `null` if it is not a hex colour. */
export function normalizeHex(text: string): string | null {
  const srgb = parseHex(text);
  return srgb === null ? null : toHex(srgb);
}

const GRADIENT_STOPS = 12;

/**
 * CSS gradient of `channel` across its range with the other channels of `color` fixed, so each
 * slider previews the colours it leads to. Built at runtime from the colour being edited.
 */
export function channelGradient(color: PickerColor, channel: ColorChannel): string {
  const { min, max } = CHANNEL_RANGES[channel];
  const stops = Array.from({ length: GRADIENT_STOPS + 1 }, (_, index) => {
    const position = index / GRADIENT_STOPS;
    const stopColor = { ...color, [channel]: min + (max - min) * position };
    return `${hexFromColor(stopColor)} ${String(position * 100)}%`;
  });
  return `linear-gradient(to right, ${stops.join(', ')})`;
}
