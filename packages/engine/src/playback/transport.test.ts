import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds } from '../testing/document';
import { FLICKS_PER_SECOND } from '../time/flicks';
import { frameIndexAt, frameStart, skippedFrames } from './frame-pacing';
import { PlaybackStatsCounter } from './playback-stats';
import { nextShuttleRate } from './shuttle';
import { playbackSegments, transportPosition, type PlaybackBounds } from './transport';

const TEN_SECONDS: PlaybackBounds = { span: { start: 0 as Flicks, end: seconds(10) }, loop: false };
const LOOP_2_TO_4: PlaybackBounds = { span: { start: seconds(2), end: seconds(4) }, loop: true };
const NTSC = { numerator: 30000, denominator: 1001 };

describe('transportPosition', () => {
  it('derives the time from the anchor and the clock', () => {
    const transport = { rate: 1, anchorTime: seconds(1), anchorClock: 100 };
    expect(transportPosition(transport, 102.5, TEN_SECONDS)).toEqual({
      time: seconds(3.5),
      ended: false,
    });
  });

  it('does not drift over ten minutes of clock', () => {
    const transport = { rate: 1, anchorTime: 0, anchorClock: 12.345678 };
    const bounds = { span: { start: 0 as Flicks, end: seconds(3600) }, loop: false };
    const position = transportPosition(transport, 12.345678 + 600, bounds);
    expect(Math.abs(position.time - 600 * FLICKS_PER_SECOND)).toBeLessThan(1);
  });

  it('stops at the edges unless looping', () => {
    expect(
      transportPosition({ rate: 2, anchorTime: seconds(8), anchorClock: 0 }, 5, TEN_SECONDS),
    ).toEqual({ time: seconds(10), ended: true });
    expect(
      transportPosition({ rate: -1, anchorTime: seconds(1), anchorClock: 0 }, 3, TEN_SECONDS),
    ).toEqual({ time: 0, ended: true });
  });

  it('wraps inside a loop span in both directions', () => {
    const forward = { rate: 1, anchorTime: seconds(3), anchorClock: 0 };
    expect(transportPosition(forward, 1.5, LOOP_2_TO_4).time).toBeCloseTo(seconds(2.5), 3);
    const reverse = { rate: -1, anchorTime: seconds(2.5), anchorClock: 0 };
    expect(transportPosition(reverse, 1, LOOP_2_TO_4).time).toBeCloseTo(seconds(3.5), 3);
  });

  it('holds the anchor when paused or before the anchor clock', () => {
    expect(transportPosition({ rate: 0, anchorTime: 7, anchorClock: 0 }, 9, TEN_SECONDS).time).toBe(
      7,
    );
    expect(transportPosition({ rate: 1, anchorTime: 7, anchorClock: 5 }, 4, TEN_SECONDS).time).toBe(
      7,
    );
  });
});

describe('playbackSegments', () => {
  it('splits a clock window at the loop wrap', () => {
    const transport = { rate: 1, anchorTime: seconds(3.9), anchorClock: 0 };
    const segments = playbackSegments(transport, LOOP_2_TO_4, 0, 0.2);
    expect(segments).toHaveLength(2);
    const [first, second] = segments;
    expect(first?.timeEnd).toBe(seconds(4));
    expect(first?.clockEnd).toBeCloseTo(0.1, 9);
    expect(second?.clockStart).toBe(first?.clockEnd);
    expect(second?.timeStart).toBe(seconds(2));
    expect(second?.timeEnd).toBeCloseTo(seconds(2.1), 0);
  });

  it('ends without looping and is empty when paused', () => {
    const transport = { rate: 1, anchorTime: seconds(9.9), anchorClock: 0 };
    const segments = playbackSegments(transport, TEN_SECONDS, 0, 1);
    expect(segments).toHaveLength(1);
    expect(segments[0]?.timeEnd).toBe(seconds(10));
    expect(playbackSegments({ ...transport, rate: 0 }, TEN_SECONDS, 0, 1)).toEqual([]);
  });

  it('chains windows without gaps', () => {
    const transport = { rate: 1, anchorTime: 0, anchorClock: 0 };
    const a = playbackSegments(transport, TEN_SECONDS, 0, 0.025);
    const b = playbackSegments(transport, TEN_SECONDS, 0.025, 0.05);
    expect(b[0]?.timeStart).toBe(a[0]?.timeEnd);
  });
});

describe('shuttle rates', () => {
  it('doubles up to 4× and restarts at 1× when the direction changes', () => {
    expect(nextShuttleRate(0, 'forward')).toBe(1);
    expect(nextShuttleRate(1, 'forward')).toBe(2);
    expect(nextShuttleRate(2, 'forward')).toBe(4);
    expect(nextShuttleRate(4, 'forward')).toBe(4);
    expect(nextShuttleRate(4, 'reverse')).toBe(-1);
    expect(nextShuttleRate(-1, 'reverse')).toBe(-2);
    expect(nextShuttleRate(-4, 'forward')).toBe(1);
  });
});

describe('frame pacing', () => {
  it('maps times to frames exactly at 29.97 fps', () => {
    const start = frameStart(45, NTSC);
    expect(frameIndexAt(start, NTSC)).toBe(45);
    expect(frameIndexAt(start - 1, NTSC)).toBe(44);
  });

  it('counts late frames skipped, not those a fast speed skips anyway', () => {
    expect(skippedFrames(10, 11, 1)).toBe(0);
    expect(skippedFrames(10, 13, 1)).toBe(2);
    expect(skippedFrames(10, 12, 2)).toBe(0);
    expect(skippedFrames(10, 7, -1)).toBe(2);
    expect(skippedFrames(50, 10, 1)).toBe(0);
  });

  it('measures frames per second and drops with a simulated clock', () => {
    const counter = new PlaybackStatsCounter();
    const rate = { numerator: 25, denominator: 1 };
    // 25 fps for one second, with frames 10 and 11 late (skipped).
    for (let index = 0; index < 25; index += 1) {
      if (index === 10 || index === 11) continue;
      counter.frameDrawn(frameStart(index, rate), rate, 1, index * 40);
    }
    expect(counter.stats(999)).toEqual({ framesPerSecond: 23, droppedFrames: 2 });
    expect(counter.stats(2500).framesPerSecond).toBe(0);
  });
});
