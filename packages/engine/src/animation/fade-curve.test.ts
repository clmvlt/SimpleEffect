import { describe, expect, it } from 'vitest';

import { LOGARITHMIC_FADE_FLOOR_DB, constantPowerWeights, fadeCurve } from './fade-curve';

describe('fade curves', () => {
  it('start silent and end full for every shape', () => {
    for (const shape of ['linear', 'constantPower', 'logarithmic'] as const) {
      expect(fadeCurve(shape, 0)).toBe(0);
      expect(fadeCurve(shape, 1)).toBe(1);
      expect(fadeCurve(shape, -0.5)).toBe(0);
      expect(fadeCurve(shape, 2)).toBe(1);
    }
  });

  it('follow their shape in between', () => {
    expect(fadeCurve('linear', 0.25)).toBe(0.25);
    expect(fadeCurve('constantPower', 0.5)).toBeCloseTo(Math.SQRT1_2, 12);
    // Even in decibels: halfway is half of the 60 dB range.
    expect(20 * Math.log10(fadeCurve('logarithmic', 0.5))).toBeCloseTo(
      LOGARITHMIC_FADE_FLOOR_DB / 2,
      9,
    );
    expect(fadeCurve('logarithmic', 0.9)).toBeCloseTo(10 ** (-6 / 20), 9);
  });

  it('cross-fade at constant power', () => {
    for (const progress of [0, 0.1, 0.5, 0.73, 1]) {
      const { outgoing, incoming } = constantPowerWeights(progress);
      expect(outgoing ** 2 + incoming ** 2).toBeCloseTo(1, 12);
    }
    expect(constantPowerWeights(0)).toEqual({ outgoing: 1, incoming: 0 });
  });
});
