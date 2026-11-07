import type { FrameRate } from '@se/protocol';

import type { Resolution } from './resolutions';

/** Select value of the « custom » resolution choice. */
export const CUSTOM_RESOLUTION = 'custom';

/** Select value of a resolution: `1920x1080`. */
export function resolutionValue({ width, height }: Resolution): string {
  return `${String(width)}x${String(height)}`;
}

/** Select value of a frame rate: `30000/1001`. */
export function frameRateValue(rate: FrameRate): string {
  return `${String(rate.numerator)}/${String(rate.denominator)}`;
}

/** The frame rate of `rates` whose select value is `value`. */
export function frameRateFromValue(
  rates: readonly FrameRate[],
  value: string,
): FrameRate | undefined {
  return rates.find((rate) => frameRateValue(rate) === value);
}
