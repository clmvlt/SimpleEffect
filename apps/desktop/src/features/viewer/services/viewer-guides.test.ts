import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { NO_GUIDES, parseViewerPreferences } from '../store/viewer-store';
import { frameGuideShapes, hasGuides } from './frame-guides';
import {
  pannedFocus,
  pictureCorners,
  pictureRect,
  shownFocus,
  type CroppedLayer,
} from './reframe-view';
import { nearestTile, tileBackground } from './sheet-tile';

const vertical = { width: 1080, height: 1920 };

describe('frame guides', () => {
  it('draws a platform’s zones and flags another aspect', () => {
    const tiktok = frameGuideShapes({ ...NO_GUIDES, safeArea: 'tiktok' }, vertical);
    expect(tiktok.zones.map((zone) => zone.id)).toEqual(['top', 'bottom', 'left', 'right']);
    expect(tiktok.mismatchedPlatform).toBeNull();
    const onLandscape = frameGuideShapes(
      { ...NO_GUIDES, safeArea: 'tiktok' },
      { width: 1920, height: 1080 },
    );
    expect(onLandscape.mismatchedPlatform).toBe('TikTok');
  });

  it('draws thirds and a centre cross sized on the shorter side', () => {
    const shapes = frameGuideShapes({ safeArea: null, thirds: true, centre: true }, vertical);
    expect(shapes.lines).toHaveLength(6);
    const [horizontalArm] = shapes.lines.slice(4);
    expect(horizontalArm).toEqual({ x1: 0.47, y1: 0.5, x2: 0.53, y2: 0.5 });
    expect(hasGuides(NO_GUIDES)).toBe(false);
    expect(hasGuides({ ...NO_GUIDES, centre: true })).toBe(true);
  });

  it('remembers guides per sequence and drops what is not guides', () => {
    const saved = JSON.stringify({
      guides: { a: { safeArea: 'youtubeShorts', thirds: true }, b: { safeArea: 'nope' }, c: 3 },
    });
    expect(parseViewerPreferences(saved).guides).toEqual({
      a: { safeArea: 'youtubeShorts', thirds: true, centre: false },
      b: { safeArea: null, thirds: false, centre: false },
    });
  });
});

describe('reframing geometry', () => {
  // A 16:9 picture filling a 9:16 frame around its left part (crop 31.6 % wide at x = 0.1).
  const layer: CroppedLayer = {
    itemId: 'filled',
    transform: {
      position: { x: 0.5, y: 0.5 },
      scale: { x: 1, y: 1 },
      rotation: 0,
      anchor: { x: 0.5, y: 0.5 },
    },
    box: vertical,
    crop: { x: 0.1, y: 0, width: 0.3164, height: 1 },
  };

  it('extends the box to the whole picture', () => {
    const rect = pictureRect(layer);
    expect(rect.width).toBeCloseTo(1080 / 0.3164, 6);
    expect(rect.x).toBeCloseTo(-0.1 * rect.width, 6);
    expect(rect.height).toBe(1920);
    const [topLeft] = pictureCorners(layer, vertical);
    expect(topLeft?.x).toBeCloseTo(rect.x, 6);
    expect(shownFocus(layer.crop).x).toBeCloseTo(0.1 + 0.3164 / 2, 9);
  });

  it('moves the kept part against the drag and stays within the picture', () => {
    const picture = pictureRect(layer).width;
    const left = pannedFocus(layer, vertical, { x: 500, y: 900 }, { x: 600, y: 900 });
    expect(left?.x).toBeCloseTo(0.1 + 0.3164 / 2 - 100 / picture, 6);
    expect(left?.y).toBe(0.5);
    const far = pannedFocus(layer, vertical, { x: 500, y: 900 }, { x: 99_999, y: 900 });
    expect(far?.x).toBeCloseTo(0.3164 / 2, 9);
  });
});

describe('sheet tiles', () => {
  const sheet = {
    tileWidth: 160,
    tileHeight: 90,
    columns: 2,
    rows: 2,
    times: [0, 100, 200, 300] as Flicks[],
    posterWidth: 1920,
    posterHeight: 1080,
  };

  it('finds the tile nearest a time and frames it over a box', () => {
    expect(nearestTile(sheet, 240 as Flicks)).toBe(2);
    expect(tileBackground(sheet, 3, 320, 180)).toEqual({
      size: '640px 360px',
      position: '-320px -180px',
    });
  });
});
