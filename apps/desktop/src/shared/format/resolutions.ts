import type { AspectRatio } from '@se/ui';

/** A frame size in pixels. */
export interface Resolution {
  readonly width: number;
  readonly height: number;
}

/** Shorter sides offered for every ratio: 720p, 1080p, 1440p, 2160p (4K). */
const SHORTER_SIDES = [720, 1080, 1440, 2160] as const;

/** Shorter side of the resolution chosen when the ratio changes. */
const DEFAULT_SHORTER_SIDE = 1080;

/** Smallest and largest frame side the core accepts (`INVALID_FORMAT` otherwise). */
export const MIN_FRAME_SIDE = 2;
export const MAX_FRAME_SIDE = 16_384;

function nearestEven(value: number): number {
  return Math.max(MIN_FRAME_SIDE, Math.round(value / 2) * 2);
}

/** The frame size of `ratio` whose shorter side is `shorterSide` (both sides even). */
export function resolutionFor(ratio: AspectRatio, shorterSide: number): Resolution {
  const landscape = ratio.width >= ratio.height;
  const longer = nearestEven(
    (shorterSide * Math.max(ratio.width, ratio.height)) / Math.min(ratio.width, ratio.height),
  );
  return landscape
    ? { width: longer, height: shorterSide }
    : { width: shorterSide, height: longer };
}

/** The resolutions offered for `ratio`, smallest first. */
export function resolutionsFor(ratio: AspectRatio): Resolution[] {
  return SHORTER_SIDES.map((side) => resolutionFor(ratio, side));
}

/** The resolution proposed when `ratio` is chosen (1080 on the shorter side). */
export function defaultResolutionFor(ratio: AspectRatio): Resolution {
  return resolutionFor(ratio, DEFAULT_SHORTER_SIDE);
}

/** Whether the core accepts this frame size: even sides between 2 and 16 384. */
export function isValidResolution({ width, height }: Resolution): boolean {
  return [width, height].every(
    (side) =>
      Number.isInteger(side) && side % 2 === 0 && side >= MIN_FRAME_SIDE && side <= MAX_FRAME_SIDE,
  );
}
