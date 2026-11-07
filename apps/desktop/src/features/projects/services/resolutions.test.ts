import { describe, expect, it } from 'vitest';

import { defaultResolutionFor, isValidResolution, resolutionsFor } from './resolutions';

describe('resolutions', () => {
  it('offers 720p to 4K for landscape and vertical ratios', () => {
    expect(resolutionsFor({ width: 16, height: 9 })).toEqual([
      { width: 1280, height: 720 },
      { width: 1920, height: 1080 },
      { width: 2560, height: 1440 },
      { width: 3840, height: 2160 },
    ]);
    expect(defaultResolutionFor({ width: 9, height: 16 })).toEqual({ width: 1080, height: 1920 });
    expect(defaultResolutionFor({ width: 4, height: 5 })).toEqual({ width: 1080, height: 1350 });
  });

  it('keeps both sides even for any ratio', () => {
    const { width, height } = defaultResolutionFor({ width: 7, height: 3 });
    expect(width % 2).toBe(0);
    expect(height).toBe(1080);
  });

  it('accepts only even sides between 2 and 16 384', () => {
    expect(isValidResolution({ width: 1920, height: 1080 })).toBe(true);
    expect(isValidResolution({ width: 1921, height: 1080 })).toBe(false);
    expect(isValidResolution({ width: 0, height: 1080 })).toBe(false);
    expect(isValidResolution({ width: 20_000, height: 1080 })).toBe(false);
  });
});
