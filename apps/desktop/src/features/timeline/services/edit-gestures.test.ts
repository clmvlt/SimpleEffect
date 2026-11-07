import type { ItemEntity, MarkerEntity } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds, testAsset, testItem, testTrack } from '../testing/fixtures';
import { gapAt, gapResizePreview, isGapOf } from './gaps';
import {
  clampTimeDelta,
  clampTrackOffset,
  dragTrackOffset,
  movedPlacements,
} from './move-placement';
import { concernedTrackIds } from './ripple-preview';
import { snapPoints, snapShift } from './snapping';
import { clampTrim, trimGroup, trimLimits, trimPreview } from './trim-placement';

const tracks = [
  testTrack('v1', 'video', ['a', 'b']),
  testTrack('v2', 'video', ['t']),
  testTrack('a1', 'audio', ['s', 'm']),
  testTrack('a2', 'audio'),
];
const a = testItem('a', 'v1', 0, 2, { linkGroup: 'g' });
const b = testItem('b', 'v1', 4, 2, { linkGroup: 'h' });
const t = testItem('t', 'v2', 1, 1);
const s = testItem('s', 'a1', 0, 2, { linkGroup: 'g' });
const m = testItem('m', 'a1', 4, 2, { linkGroup: 'h' });
const items = [a, b, t, s, m];
const itemsByTrack = new Map(
  tracks.map((track) => [track.id, items.filter((item) => item.trackId === track.id)]),
);

describe('snapping', () => {
  it('lists the playhead, markers, in/out and edges of other items', () => {
    const marker = { id: 'k', sequenceId: 'sequence-1', time: seconds(9) } as MarkerEntity;
    const points = snapPoints({
      items,
      exclude: new Set(['a', 's']),
      playhead: seconds(7),
      markers: [marker],
      inOut: [seconds(8)],
    });
    expect(points).toEqual([0, 1, 2, 4, 6, 7, 8, 9].map(seconds));
  });

  it('shifts the closest edge onto a point within the threshold', () => {
    const points = [0, 4, 10].map(seconds);
    expect(snapShift([seconds(3.9), seconds(5.9)], points, seconds(0.2))).toEqual({
      delta: seconds(0.1),
      point: seconds(4),
    });
    expect(snapShift([seconds(7)], points, seconds(0.2))).toBeNull();
  });
});

describe('move placements', () => {
  it('moves picture and sound by the same number of tracks of their kind', () => {
    expect(dragTrackOffset(tracks, 'v1', 'v2', 0)).toBe(1);
    expect(dragTrackOffset(tracks, 'v1', 'a1', 1)).toBe(1);
    expect(clampTrackOffset(tracks, [a, s], 1)).toBe(1);
    expect(clampTrackOffset(tracks, [t], 1)).toBe(0);
    expect(movedPlacements([a, s], tracks, seconds(3), 1)).toEqual([
      { itemId: 'a', trackId: 'v2', start: seconds(3) },
      { itemId: 's', trackId: 'a2', start: seconds(3) },
    ]);
  });

  it('never moves items before zero', () => {
    expect(clampTimeDelta([b, t], seconds(-3))).toBe(seconds(-1));
  });
});

describe('trims', () => {
  const context = {
    itemsByTrack,
    assets: { 'asset-1': testAsset('asset-1', '/m.mp4') },
    frame: seconds(0.04),
    ripple: false,
  };

  it('trims linked partners whose edge sits at the same instant, alone with Alt', () => {
    expect(trimGroup(a, 'end', items, false).map((item) => item.id)).toEqual(['a', 's']);
    expect(trimGroup(a, 'end', items, true).map((item) => item.id)).toEqual(['a']);
  });

  it('stops at neighbours, keeps one frame and stays inside the media', () => {
    expect(trimLimits([a], 'end', context)).toEqual({ min: seconds(0.04), max: seconds(4) });
    expect(trimLimits([a], 'end', { ...context, ripple: true }).max).toBe(Infinity);
    const clip: ItemEntity = testItem('c', 'v1', 10, 5, {
      kind: {
        kind: 'mediaClip',
        assetId: 'asset-1',
        sourceStart: seconds(2),
        speed: 1,
      } as ItemEntity['kind'],
    });
    const media = { ...context, itemsByTrack: new Map([['v1', [clip]]]) };
    expect(trimLimits([clip], 'start', media).min).toBe(seconds(8));
    expect(trimLimits([clip], 'end', media).max).toBe(seconds(38));
    expect(clampTrim(seconds(50), { min: seconds(8), max: seconds(38) }, seconds(15))).toBe(
      seconds(38),
    );
  });

  it('previews a ripple trim by moving what follows on the linked tracks', () => {
    const preview = trimPreview([a, s], 'end', seconds(1), true, tracks, itemsByTrack);
    expect(preview.get('a')).toEqual({ trackId: 'v1', start: 0, duration: seconds(1) });
    expect(preview.get('b')?.start).toBe(seconds(3));
    expect(preview.get('m')?.start).toBe(seconds(3));
    expect(preview.has('t')).toBe(false);
    const start = trimPreview([b], 'start', seconds(5), true, tracks, itemsByTrack);
    expect(start.get('b')).toEqual({ trackId: 'v1', start: seconds(4), duration: seconds(1) });
  });
});

describe('ripple tracks and gaps', () => {
  it('adds the tracks of items linked to what moves, never locked ones', () => {
    expect(concernedTrackIds(tracks, itemsByTrack, new Set(['v1']), seconds(3))).toEqual(
      new Set(['v1', 'a1']),
    );
    const locked = tracks.map((track) => (track.id === 'a1' ? { ...track, locked: true } : track));
    expect(concernedTrackIds(locked, itemsByTrack, new Set(['v1']), seconds(3))).toEqual(
      new Set(['v1']),
    );
  });

  it('finds the gap under a time, not after the last item', () => {
    const v1 = itemsByTrack.get('v1') ?? [];
    expect(gapAt(v1, 'v1', seconds(3))).toEqual({
      trackId: 'v1',
      start: seconds(2),
      end: seconds(4),
    });
    expect(gapAt(v1, 'v1', seconds(1))).toBeNull();
    expect(gapAt(v1, 'v1', seconds(9))).toBeNull();
    expect(isGapOf(v1, { trackId: 'v1', start: seconds(2), end: seconds(4) })).toBe(true);
    expect(isGapOf(v1, { trackId: 'v1', start: seconds(2), end: seconds(5) })).toBe(false);
  });

  it('previews a gap resize by moving what follows on the linked tracks', () => {
    const gap = { trackId: 'v1', start: seconds(2), end: seconds(4) };
    const preview = gapResizePreview(gap, seconds(5), tracks, itemsByTrack);
    expect(preview.get('b')?.start).toBe(seconds(7));
    expect(preview.get('m')?.start).toBe(seconds(7));
    expect(preview.has('t')).toBe(false);
  });
});
