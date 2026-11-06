import { describe, expect, it } from 'vitest';

import {
  clampToRange,
  decimalsOf,
  incrementFor,
  parseScrubValue,
  snapToIncrement,
  stepValue,
} from './scrub-math';

const SPEEDS = { fast: 10, fine: 0.1 };

describe('scrub math', () => {
  it('counts the decimals of a step', () => {
    expect(decimalsOf(1)).toBe(0);
    expect(decimalsOf(0.25)).toBe(2);
    expect(decimalsOf(0.1 * 0.1)).toBe(2);
    expect(decimalsOf(1e-3)).toBe(3);
  });

  it('clamps with open bounds', () => {
    expect(clampToRange(5, { min: 0, max: 3 })).toBe(3);
    expect(clampToRange(-5, { min: 0 })).toBe(0);
    expect(clampToRange(1e9, {})).toBe(1e9);
  });

  it('picks the increment from the modifiers', () => {
    expect(incrementFor(2, { shiftKey: false, altKey: false }, SPEEDS)).toBe(2);
    expect(incrementFor(2, { shiftKey: true, altKey: false }, SPEEDS)).toBe(20);
    expect(incrementFor(2, { shiftKey: false, altKey: true }, SPEEDS)).toBeCloseTo(0.2);
  });

  it('snaps without binary noise', () => {
    expect(snapToIncrement(0.30000000000000004, 0.1)).toBe(0.3);
    expect(snapToIncrement(7.4, 5)).toBe(5);
  });

  it('steps repeatedly without accumulating float noise', () => {
    let value = 0;
    for (let index = 0; index < 3; index += 1) {
      value = stepValue(value, 1, 0.1, { step: 1 }, SPEEDS);
    }
    expect(value).toBe(0.3);
  });

  it('parses typed numbers with a comma or a unit', () => {
    expect(parseScrubValue('12,5')).toBe(12.5);
    expect(parseScrubValue(' -3.25 px ')).toBe(-3.25);
    expect(parseScrubValue('.5')).toBe(0.5);
    expect(parseScrubValue('90°')).toBe(90);
    expect(parseScrubValue('abc')).toBeNull();
    expect(parseScrubValue('')).toBeNull();
    expect(parseScrubValue('1 2')).toBeNull();
  });
});
