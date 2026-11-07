import type { Format, FrameRate } from '@se/protocol';
import { ASPECT_RATIO_PRESETS, formatAspectRatio, type AspectRatio } from '@se/ui';

function greatestCommonDivisor(a: number, b: number): number {
  return b === 0 ? a : greatestCommonDivisor(b, a % b);
}

/** The reduced aspect ratio of a frame size (`1920 × 1080` → `16:9`). */
export function aspectRatioOf(width: number, height: number): AspectRatio {
  const divisor = greatestCommonDivisor(width, height) || 1;
  return { width: width / divisor, height: height / divisor };
}

/**
 * The badge shown on a project card: the ratio when it is a common one (`16:9`, `9:16`, `1:1`,
 * `4:5`), the frame size otherwise (`1280×544`).
 */
export function formatBadgeLabel(format: Format): string {
  const ratio = aspectRatioOf(format.width, format.height);
  const common = ASPECT_RATIO_PRESETS.some(
    (preset) => preset.width === ratio.width && preset.height === ratio.height,
  );
  return common ? formatAspectRatio(ratio) : `${String(format.width)}×${String(format.height)}`;
}

/** Frames per second as a number (`30000/1001` → `29.97…`), for display only. */
export function framesPerSecond(rate: FrameRate): number {
  return rate.numerator / rate.denominator;
}

/** Whether two frame rates are the same fraction (rates arrive reduced from the core). */
export function sameFrameRate(a: FrameRate, b: FrameRate): boolean {
  return a.numerator === b.numerator && a.denominator === b.denominator;
}
