import { describe, expect, it } from 'vitest';

import { aspectRatioOf, formatBadgeLabel, framesPerSecond } from './format-label';

const format = (width: number, height: number) => ({
  width,
  height,
  frameRate: { numerator: 30, denominator: 1 },
  sampleRate: 48_000,
});

describe('format labels', () => {
  it('reduces ratios', () => {
    expect(aspectRatioOf(3840, 2160)).toEqual({ width: 16, height: 9 });
    expect(aspectRatioOf(1080, 1350)).toEqual({ width: 4, height: 5 });
  });

  it('shows common ratios and the size of others', () => {
    expect(formatBadgeLabel(format(1920, 1080))).toBe('16:9');
    expect(formatBadgeLabel(format(1080, 1920))).toBe('9:16');
    expect(formatBadgeLabel(format(1280, 544))).toBe('1280×544');
  });

  it('turns NTSC fractions into frames per second', () => {
    expect(framesPerSecond({ numerator: 30_000, denominator: 1001 })).toBeCloseTo(29.97, 2);
  });
});
