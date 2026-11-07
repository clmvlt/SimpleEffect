import type { FrameRate } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds } from '../testing/fixtures';
import { majorStep, rulerTicks } from './ruler-ticks';
import {
  clampScroll,
  DEFAULT_PIXELS_PER_SECOND,
  fitViewport,
  maxScrollTime,
  pageTo,
  revealTime,
  scrollByPixels,
  sliderToZoom,
  timeToX,
  visibleRange,
  xToTime,
  zoomAround,
  zoomLimits,
  zoomToSlider,
  type Viewport,
} from './viewport';

const PAL: FrameRate = { numerator: 25, denominator: 1 };
const NTSC: FrameRate = { numerator: 30000, denominator: 1001 };
const limits = zoomLimits(PAL);
const view = (pixelsPerSecond: number, scroll: number, width = 1000): Viewport => ({
  pixelsPerSecond,
  scrollTime: seconds(scroll),
  width,
});

describe('time ↔ pixels in a viewport', () => {
  it('places times relative to the scroll', () => {
    expect(timeToX(seconds(12), view(100, 10))).toBe(200);
    expect(xToTime(200, view(100, 10))).toBe(seconds(12));
    expect(xToTime(-5000, view(100, 10))).toBe(0);
  });

  it('widens the visible range by the overscan', () => {
    expect(visibleRange(view(100, 10), 100)).toEqual({ start: seconds(9), end: seconds(21) });
  });
});

describe('zoom', () => {
  it('keeps the time under the anchor in place', () => {
    const before = view(100, 10);
    const after = zoomAround(before, 400, 300, limits);
    expect(after.pixelsPerSecond).toBe(400);
    expect(xToTime(300, after)).toBe(xToTime(300, before));
  });

  it('stays within the limits (120 px per frame at most)', () => {
    expect(zoomAround(view(100, 0), 1e9, 0, limits).pixelsPerSecond).toBe(3000);
    expect(zoomAround(view(100, 0), 0, 0, limits).pixelsPerSecond).toBe(0.01);
  });

  it('fits the content in 95 % of the width, or uses the default zoom when empty', () => {
    expect(fitViewport(view(10, 50), seconds(95), limits)).toEqual(view(10, 0));
    expect(fitViewport(view(10, 50), seconds(0), limits).pixelsPerSecond).toBe(
      DEFAULT_PIXELS_PER_SECOND,
    );
  });

  it('maps the slider logarithmically, both ways', () => {
    expect(zoomToSlider(limits.min, limits)).toBe(0);
    expect(zoomToSlider(limits.max, limits)).toBe(1000);
    expect(sliderToZoom(zoomToSlider(80, limits), limits)).toBeCloseTo(80, -1);
  });
});

describe('scroll', () => {
  it('stops half a width after the content', () => {
    expect(maxScrollTime(view(100, 0), seconds(30))).toBe(seconds(25));
    expect(clampScroll(view(100, 90), seconds(30)).scrollTime).toBe(seconds(25));
    expect(scrollByPixels(view(100, 0), -500, seconds(30)).scrollTime).toBe(0);
    expect(scrollByPixels(view(100, 0), 500, seconds(30)).scrollTime).toBe(seconds(5));
  });

  it('reveals a time out of view, leaving a visible one alone', () => {
    const shown = view(100, 10);
    expect(revealTime(shown, seconds(15), seconds(60))).toBe(shown);
    expect(revealTime(shown, seconds(40), seconds(60)).scrollTime).toBe(seconds(31));
    expect(revealTime(shown, seconds(5), seconds(60)).scrollTime).toBe(seconds(4));
  });

  it('pages while playing: the playhead restarts near the leading edge', () => {
    const shown = view(100, 10);
    expect(pageTo(shown, seconds(19), 1, seconds(60))).toBe(shown);
    expect(pageTo(shown, seconds(21), 1, seconds(60)).scrollTime).toBe(seconds(20.5));
    expect(pageTo(view(100, 30), seconds(25), -1, seconds(60)).scrollTime).toBe(seconds(15.5));
  });
});

describe('ruler ticks', () => {
  it('keeps labels at least 96 px apart on round timecodes', () => {
    const ticks = rulerTicks(view(20, 0), PAL);
    expect(ticks.major.slice(0, 3).map((tick) => tick.label)).toEqual([
      '00:00:00:00',
      '00:00:05:00',
      '00:00:10:00',
    ]);
    expect(ticks.major[1]?.x).toBe(100);
    expect(ticks.minor.length).toBeGreaterThan(0);
  });

  it('labels frames when zoomed in and counts nominal frames at 29.97', () => {
    expect(majorStep(view(3000, 0), PAL)).toBe(1);
    const ntsc = rulerTicks(view(100, 0), NTSC);
    expect(ntsc.major[1]?.label).toBe('00:00:01:00');
    expect(ntsc.major[1]?.x).toBeCloseTo(100.1, 1);
  });

  it('starts at the first label before the scroll', () => {
    const ticks = rulerTicks(view(20, 7), PAL);
    expect(ticks.major[0]?.label).toBe('00:00:05:00');
    expect(ticks.major[0]?.x).toBe(-40);
    expect(ticks.major.every((tick) => tick.x <= 1000)).toBe(true);
  });
});
