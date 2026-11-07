import type { FadeShape } from '@se/protocol';

/** Level, in decibels, a logarithmic fade starts from (its curve is even in decibels). */
export const LOGARITHMIC_FADE_FLOOR_DB = -60;

/**
 * Level of a fade at `progress` (0 = silent or black, 1 = full), for the domain's fade shapes
 * (`FadeShape`): the gain of a clip's sound and the factor of its picture's opacity. Progress is
 * clamped to `[0, 1]`; the curve is exactly 0 at 0 and 1 at 1.
 */
export function fadeCurve(shape: FadeShape, progress: number): number {
  const clamped = Math.min(1, Math.max(0, progress));
  if (clamped === 0 || clamped === 1) return clamped;
  switch (shape) {
    case 'linear':
      return clamped;
    case 'constantPower':
      return Math.sin((clamped * Math.PI) / 2);
    case 'logarithmic':
      return 10 ** ((LOGARITHMIC_FADE_FLOOR_DB * (1 - clamped)) / 20);
  }
}

/**
 * Weights of the outgoing and incoming sides of a sound cross-fade at `progress`: constant power
 * (`cos` and `sin` of a quarter turn), so the loudness of uncorrelated sounds holds steady.
 */
export function constantPowerWeights(progress: number): {
  readonly outgoing: number;
  readonly incoming: number;
} {
  const clamped = Math.min(1, Math.max(0, progress));
  return {
    outgoing: Math.cos((clamped * Math.PI) / 2),
    incoming: Math.sin((clamped * Math.PI) / 2),
  };
}
