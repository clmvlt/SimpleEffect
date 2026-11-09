import type { ThumbnailSheet, WaveformPeaks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { transform } from '../../../shared/project-store/testing/graphics-fixture';
import { seconds, testAsset, testItem } from '../testing/fixtures';
import { contentEnd, editPoints, nextEditPoint, previousEditPoint, snapTime } from './edit-points';
import { itemBox, OVERSCAN_PIXELS, spanBox } from './item-geometry';
import { fileName, isAiMade, itemDisplay } from './item-display';
import { stripTiles, tileBackground } from './thumbnail-strip';
import type { Viewport } from './viewport';
import { visibleItems } from './visible-items';
import { peakLevelFor, waveformColumns } from './waveform';

const view: Viewport = { pixelsPerSecond: 100, scrollTime: seconds(10), width: 1000 };

describe('virtualisation', () => {
  const items = Array.from({ length: 500 }, (_, index) =>
    testItem(`i${String(index)}`, 'v1', index * 2, 1.5),
  );

  it('returns only the items crossing the range, found by binary search', () => {
    const shown = visibleItems(items, seconds(100), seconds(110));
    expect(shown.map((item) => item.id)).toEqual(['i50', 'i51', 'i52', 'i53', 'i54']);
  });

  it('includes an item overlapping the start and excludes one touching the end', () => {
    expect(visibleItems(items, seconds(101), seconds(104)).map((item) => item.id)).toEqual([
      'i50',
      'i51',
    ]);
    expect(visibleItems(items, seconds(1.5), seconds(2))).toEqual([]);
    expect(visibleItems([], seconds(0), seconds(10))).toEqual([]);
  });
});

describe('item geometry', () => {
  it('draws only the visible part of a long clip and remembers the hidden offset', () => {
    const box = itemBox(testItem('a', 'v1', 0, 60), view);
    expect(box.left).toBe(-OVERSCAN_PIXELS);
    expect(box.width).toBe(1000 + 2 * OVERSCAN_PIXELS);
    expect(box.offsetPixels).toBe(1000 - OVERSCAN_PIXELS);
  });

  it('keeps tiny clips one pixel wide and boxes spans', () => {
    expect(itemBox(testItem('a', 'v1', 12, 0.001), view).width).toBe(1);
    expect(spanBox({ start: seconds(11), end: seconds(13) }, view)).toEqual({
      left: 100,
      width: 200,
    });
  });
});

describe('edit points', () => {
  const items = [testItem('a', 'v1', 1, 2), testItem('b', 'v2', 3, 1), testItem('c', 'v1', 6, 1)];

  it('lists the start and every item edge once, sorted', () => {
    const points = editPoints(items);
    expect(points).toEqual([0, 1, 3, 4, 6, 7].map(seconds));
    expect(previousEditPoint(points, seconds(3))).toBe(seconds(1));
    expect(nextEditPoint(points, seconds(3))).toBe(seconds(4));
    expect(nextEditPoint(points, seconds(7))).toBeUndefined();
    expect(contentEnd(items)).toBe(seconds(7));
    expect(contentEnd([])).toBe(0);
  });

  it('snaps to the nearest candidate within the threshold only', () => {
    const candidates = [seconds(1), seconds(3)];
    expect(snapTime(seconds(2.95), candidates, seconds(0.1))).toBe(seconds(3));
    expect(snapTime(seconds(2), candidates, seconds(0.1))).toBe(seconds(2));
  });
});

describe('waveforms', () => {
  it('picks the coarsest peak level that keeps a peak per pixel', () => {
    expect(peakLevelFor(48_000, 1, 10)).toBe(4096);
    expect(peakLevelFor(48_000, 1, 100)).toBe(256);
    expect(peakLevelFor(48_000, 1, 40)).toBe(1024);
    expect(peakLevelFor(48_000, 1, 5000)).toBe(256);
  });

  it('takes the extremes of the blocks under each column, all channels', () => {
    const peaks: WaveformPeaks = {
      channels: 2,
      sampleRate: 1000,
      samplesPerPeak: 100,
      peakCount: 4,
      // Blocks of 0.1 s: [min, max] for channel 0 then channel 1.
      values: Int16Array.from([
        -1000, 1000, -2000, 500, -16384, 16384, 0, 0, 0, 0, -8192, 32767, -100, 100, -100, 100,
      ]),
    };
    const columns = waveformColumns(peaks, {
      sourceStartSeconds: 0,
      speed: 1,
      pixelsPerSecond: 5,
      offsetPixels: 0,
      width: 3,
    });
    expect(Array.from(columns.minimums)).toEqual([-0.5, -0.25, 0]);
    expect(columns.maximums[0]).toBe(0.5);
    expect(columns.maximums[1]).toBeCloseTo(1, 3);
    expect(columns.maximums[2]).toBe(0);
  });
});

describe('thumbnail strips', () => {
  const sheet: ThumbnailSheet = {
    tileWidth: 160,
    tileHeight: 90,
    columns: 8,
    rows: 2,
    times: [0, 2, 4, 6, 8, 10, 12, 14, 16, 18].map(seconds),
    posterWidth: 640,
    posterHeight: 360,
  };

  it('lines tiles up from the clip start at the lane height, showing the right source time', () => {
    const tiles = stripTiles(sheet, {
      sourceStart: seconds(4),
      speed: 1,
      pixelsPerSecond: 20,
      offsetPixels: 100,
      width: 200,
      height: 45,
    });
    expect(tiles.map((tile) => tile.left)).toEqual([-20, 60, 140]);
    expect(tiles.map((tile) => tile.index)).toEqual([4, 6, 8]);
    expect(tiles[0]?.width).toBe(80);
  });

  it('cuts the tile out of the sheet', () => {
    expect(tileBackground(sheet, { left: 0, width: 80, index: 9 }, 45)).toEqual({
      size: '640px 90px',
      position: '-80px -45px',
    });
  });
});

describe('item display', () => {
  const assets = {
    clip: testAsset('clip', 'C:\\rushes\\Plage.mp4'),
    song: testAsset('song', '/music/Été.wav', 'audio'),
  };
  const media = (assetId: string) =>
    testItem('m', 'v1', 0, 1, {
      kind: {
        kind: 'mediaClip',
        assetId,
        sourceStart: seconds(0),
        speed: 1,
        transform: null as never,
        volumeDb: { kind: 'static', value: 0 },
        fadeIn: seconds(0),
        fadeOut: seconds(0),
        fadeInShape: 'linear',
        fadeOutShape: 'linear',
        reframe: { kind: 'fit' },
      },
    });

  it('shows thumbnails on video tracks and waveforms on audio tracks, named after the file', () => {
    expect(itemDisplay(media('clip'), 'video', assets, [])).toMatchObject({
      tone: 'video',
      picture: 'thumbnails',
      label: 'Plage.mp4',
    });
    expect(itemDisplay(media('clip'), 'audio', assets, [])).toMatchObject({
      tone: 'audio',
      picture: 'waveform',
    });
    expect(itemDisplay(media('gone'), 'video', assets, [])).toMatchObject({
      label: null,
      fallback: 'media',
    });
  });

  it('labels texts, captions and templates by their content', () => {
    const text = testItem('t', 'v1', 0, 1, {
      kind: {
        kind: 'text',
        content: ' Titre ',
        style: null as never,
        transform: null as never,
        reveal: null as never,
      },
    });
    expect(itemDisplay(text, 'video', assets, [])).toMatchObject({
      tone: 'graphic',
      label: 'Titre',
    });
    expect(itemDisplay(testItem('c', 'v1', 0, 1), 'video', assets, [])).toMatchObject({
      tone: 'caption',
      label: 'Bonjour',
    });
    const template = testItem('p', 'v1', 0, 1, {
      kind: { kind: 'motionTemplate', templateId: 'lower', params: {}, transform: transform() },
    });
    expect(itemDisplay(template, 'video', assets, [{ id: 'lower', name: 'Bandeau' }]).label).toBe(
      'Bandeau',
    );
  });

  it('knows file names and AI origins', () => {
    expect(fileName('C:\\a\\b.mov')).toBe('b.mov');
    expect(fileName('/a/b.wav')).toBe('b.wav');
    expect(
      isAiMade(testItem('a', 'v1', 0, 1, { origin: { kind: 'aiTask', taskId: 'task' } })),
    ).toBe(true);
  });
});
