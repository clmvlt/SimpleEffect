import { describe, expect, it } from 'vitest';

import { SAFE_AREA_PLATFORMS, matchesAspect, normalisePlatform, safeAreaPlatform } from './index';

describe('safe areas', () => {
  it('lists every platform once with its source and measurement date', () => {
    expect(SAFE_AREA_PLATFORMS.map((platform) => platform.id)).toEqual([
      'tiktok',
      'instagramReels',
      'youtubeShorts',
      'youtube16x9',
    ]);
    for (const platform of SAFE_AREA_PLATFORMS) {
      expect(platform.source.length, platform.id).toBeGreaterThan(20);
      expect(platform.url, platform.id).toMatch(/^https:\/\//);
      expect(platform.measuredOn, platform.id).toMatch(/^\d{4}-\d{2}-\d{2}$/);
      expect(platform.zones.length, platform.id).toBeGreaterThan(0);
    }
  });

  it('keeps every zone inside the frame', () => {
    for (const platform of SAFE_AREA_PLATFORMS) {
      for (const { id, rect } of platform.zones) {
        const where = `${platform.id}.${id}`;
        expect(rect.x, where).toBeGreaterThanOrEqual(0);
        expect(rect.y, where).toBeGreaterThanOrEqual(0);
        expect(rect.x + rect.width, where).toBeLessThanOrEqual(1 + 1e-9);
        expect(rect.y + rect.height, where).toBeLessThanOrEqual(1 + 1e-9);
      }
    }
  });

  it('normalises pixels of the reference frame', () => {
    const tiktok = safeAreaPlatform('tiktok');
    const top = tiktok?.zones.find((zone) => zone.id === 'top');
    expect(top?.rect).toEqual({ x: 0, y: 0, width: 1, height: 0.125 });
    const title = safeAreaPlatform('youtube16x9')?.zones.find((zone) => zone.id === 'title');
    expect(title?.rect.x).toBeCloseTo(0.05, 9);
    expect(title?.rect.width).toBeCloseTo(0.9, 9);
  });

  it('tells whether a frame has the measured aspect', () => {
    const shorts = safeAreaPlatform('youtubeShorts');
    expect(shorts && matchesAspect(shorts, { width: 1080, height: 1920 })).toBe(true);
    expect(shorts && matchesAspect(shorts, { width: 1920, height: 1080 })).toBe(false);
  });

  it('refuses unknown kinds', () => {
    expect(() =>
      normalisePlatform({
        id: 'tiktok',
        name: 'TikTok',
        aspect: { width: 9, height: 16 },
        reference: { width: 1080, height: 1920 },
        source: 'test',
        url: 'https://example.test',
        measuredOn: '2026-11-07',
        zones: [{ id: 'top', kind: 'hidden', rect: { x: 0, y: 0, width: 1, height: 1 } }],
      }),
    ).toThrow(/zone kind 'hidden'/);
  });
});
