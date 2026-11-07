import type { TrackKind } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds, testItem, testTrack } from '../testing/fixtures';
import { inOutSpan, NO_IN_OUT, pointsBetween, withInPoint, withOutPoint } from './in-out';
import { itemsInRect, marqueeBox, marqueeSelection } from './marquee';
import {
  overviewBars,
  overviewSeconds,
  viewportFromWindow,
  windowPercent,
  windowRect,
} from './minimap';
import { linkedItems, nextSelection, selectionModeOf } from './selection';
import { clampTrackHeight, wheelResizedHeight } from './track-height';
import {
  dropIndex,
  dropLineTop,
  insertionIndex,
  layoutTracks,
  reorderedTrackIds,
  rowAt,
  SECTION_DIVIDER_HEIGHT,
} from './track-layout';
import { zoomLimits, type Viewport } from './viewport';

// Document order: video tracks bottom-up, then audio.
const tracks = [
  testTrack('v1', 'video'),
  testTrack('v2', 'video'),
  testTrack('a1', 'audio'),
  testTrack('a2', 'audio'),
];
const ids = tracks.map((track) => track.id);
const kindOf = (id: string): TrackKind | undefined => tracks.find((track) => track.id === id)?.kind;
const layout = layoutTracks(tracks, (track) => (track.id === 'v2' ? 80 : 40));

describe('track layout', () => {
  it('stacks video tracks top-down in reverse, then the divider, then audio tracks', () => {
    expect(layout.video.map((row) => [row.track.id, row.top])).toEqual([
      ['v2', 0],
      ['v1', 80],
    ]);
    expect(layout.dividerTop).toBe(120);
    expect(layout.audio.map((row) => [row.track.id, row.top])).toEqual([
      ['a1', 120 + SECTION_DIVIDER_HEIGHT],
      ['a2', 160 + SECTION_DIVIDER_HEIGHT],
    ]);
    expect(layout.height).toBe(200 + SECTION_DIVIDER_HEIGHT);
    expect(rowAt(layout, 90)?.track.id).toBe('v1');
    expect(rowAt(layout, 122)).toBeUndefined();
  });

  it('inserts « above » a video track after it and « above » an audio track before it', () => {
    const [v1, , a1] = tracks;
    if (v1 === undefined || a1 === undefined) throw new Error('fixture');
    expect(insertionIndex(ids, v1, 'above')).toBe(1);
    expect(insertionIndex(ids, v1, 'below')).toBe(0);
    expect(insertionIndex(ids, a1, 'above')).toBe(2);
    expect(insertionIndex(ids, a1, 'below')).toBe(3);
  });

  it('reorders within a kind and keeps the other kind in place', () => {
    // On screen the video tracks read v2, v1: moving v1 to the top puts it last in the list.
    expect(reorderedTrackIds(ids, kindOf, 'v1', 0)).toEqual(['v2', 'v1', 'a1', 'a2']);
    expect(reorderedTrackIds(ids, kindOf, 'a2', 0)).toEqual(['v1', 'v2', 'a2', 'a1']);
    const interleaved = ['v1', 'a1', 'v2', 'a2'];
    expect(reorderedTrackIds(interleaved, kindOf, 'v2', 1)).toEqual(['v2', 'a1', 'v1', 'a2']);
  });

  it('finds where a dragged track lands and where to draw the line', () => {
    expect(dropIndex(layout.video, 'v2', 110)).toBe(1);
    expect(dropIndex(layout.video, 'v2', 10)).toBe(0);
    expect(dropLineTop(layout.video, 'v2', 1)).toBe(120);
    expect(dropLineTop(layout.video, 'v2', 0)).toBe(80);
  });

  it('bounds track heights and resizes them by wheel steps', () => {
    expect(clampTrackHeight(5)).toBe(24);
    expect(clampTrackHeight(1000)).toBe(200);
    expect(wheelResizedHeight(50, -100)).toBe(56);
    expect(wheelResizedHeight(50, 100)).toBe(44);
  });
});

describe('selection', () => {
  it('reads the mode from the modifiers of the platform', () => {
    const click = { shiftKey: false, ctrlKey: false, metaKey: false };
    expect(selectionModeOf(click, false)).toBe('replace');
    expect(selectionModeOf({ ...click, shiftKey: true }, false)).toBe('add');
    expect(selectionModeOf({ ...click, ctrlKey: true }, false)).toBe('toggle');
    expect(selectionModeOf({ ...click, metaKey: true }, true)).toBe('toggle');
    expect(selectionModeOf({ ...click, ctrlKey: true }, true)).toBe('replace');
  });

  it('replaces, adds or toggles the clicked items', () => {
    expect(nextSelection(['a', 'b'], ['c'], 'replace')).toEqual(['c']);
    expect(nextSelection(['a'], ['b', 'c'], 'add')).toEqual(['a', 'b', 'c']);
    expect(nextSelection(['a', 'b'], ['b'], 'toggle')).toEqual(['a']);
    expect(nextSelection(['a'], ['b', 'c'], 'toggle')).toEqual(['a', 'b', 'c']);
  });

  it('selects linked items together unless Alt is held', () => {
    const picture = testItem('p', 'v1', 0, 1, { linkGroup: 'g' });
    const sound = testItem('s', 'a1', 0, 1, { linkGroup: 'g' });
    const other = testItem('o', 'a1', 2, 1);
    const all = { p: picture, s: sound, o: other };
    expect(linkedItems(picture, all, ['p', 's', 'o'], false)).toEqual(['p', 's']);
    expect(linkedItems(picture, all, ['p', 's', 'o'], true)).toEqual(['p']);
    expect(linkedItems(other, all, ['p', 's', 'o'], false)).toEqual(['o']);
  });

  it('selects what a rectangle touches, combined with the previous selection', () => {
    const view: Viewport = { pixelsPerSecond: 100, scrollTime: seconds(0), width: 1000 };
    const items = [
      testItem('v2a', 'v2', 0, 1),
      testItem('v2b', 'v2', 5, 1),
      testItem('v1a', 'v1', 0.5, 1),
      testItem('a1a', 'a1', 0, 1),
    ];
    const rect = { x0: 250, y0: 100, x1: 20, y1: 10 };
    expect(marqueeBox(rect)).toEqual({ left: 20, top: 10, width: 230, height: 90 });
    const touched = itemsInRect(rect, layout, view, items);
    expect(touched).toEqual(['v2a', 'v1a']);
    expect(marqueeSelection(['a1a'], touched, 'add')).toEqual(['a1a', 'v2a', 'v1a']);
    expect(marqueeSelection(['v2a'], touched, 'toggle')).toEqual(['v1a']);
  });
});

describe('in and out points', () => {
  it('drops a point the other one would cross', () => {
    const marked = withOutPoint(withInPoint(NO_IN_OUT, seconds(2)), seconds(5));
    expect(marked).toEqual({ inPoint: seconds(2), outPoint: seconds(5) });
    expect(withInPoint(marked, seconds(6))).toEqual({ inPoint: seconds(6), outPoint: null });
    expect(withOutPoint(marked, seconds(1))).toEqual({ inPoint: null, outPoint: seconds(1) });
  });

  it('spans a drag in any direction and extends a lone point to the content', () => {
    expect(pointsBetween(seconds(5), seconds(2))).toEqual({
      inPoint: seconds(2),
      outPoint: seconds(5),
    });
    expect(pointsBetween(seconds(5), seconds(5))).toEqual(NO_IN_OUT);
    expect(inOutSpan({ inPoint: seconds(3), outPoint: null }, seconds(10))).toEqual({
      start: seconds(3),
      end: seconds(10),
    });
    expect(inOutSpan({ inPoint: null, outPoint: seconds(4) }, seconds(10))).toEqual({
      start: 0,
      end: seconds(4),
    });
    expect(inOutSpan(NO_IN_OUT, seconds(10))).toBeNull();
    expect(inOutSpan({ inPoint: seconds(12), outPoint: null }, seconds(10))).toBeNull();
  });
});

describe('minimap', () => {
  const view: Viewport = { pixelsPerSecond: 10, scrollTime: seconds(20), width: 500 };
  const limits = zoomLimits({ numerator: 25, denominator: 1 });

  it('shows the content and the visible window', () => {
    expect(overviewSeconds(view, seconds(100))).toBe(105);
    expect(overviewSeconds(view, seconds(10))).toBe(70);
    const rect = windowRect(view, seconds(100), 1050);
    expect(rect).toEqual({ left: 200, width: 500 });
    expect(windowPercent(rect, 1050)).toBe(36);
  });

  it('scrolls when the window moves and zooms when an edge moves', () => {
    const moved = viewportFromWindow(view, { left: 400, right: 900 }, seconds(100), 1050, limits);
    expect(moved.scrollTime).toBe(seconds(40));
    expect(moved.pixelsPerSecond).toBeCloseTo(10);
    const widened = viewportFromWindow(
      view,
      { left: 200, right: 1200 },
      seconds(100),
      1050,
      limits,
    );
    expect(widened.pixelsPerSecond).toBeCloseTo(5);
  });

  it('draws one bar per item on its track line', () => {
    const rows = layoutTracks([testTrack('v1', 'video', ['x'])], () => 40).video;
    const bars = overviewBars(
      rows,
      new Map([['v1', [testItem('x', 'v1', 10, 20)]]]),
      100,
      1000,
      20,
    );
    expect(bars).toEqual([{ x: 100, y: 2, width: 200, kind: 'video' }]);
  });
});
