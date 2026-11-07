import type { Flicks, FrameRate } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { clampFlicks, FLICKS_PER_SECOND, snapToFrame } from './flicks';
import { formatFrameRate } from './frame-rate';
import { divideRounded } from './rounding';
import { flicksToPixels, pixelsToFlicks } from './time-scale';
import { formatTimecode, parseTimecode } from './timecode';

const NTSC: FrameRate = { numerator: 30000, denominator: 1001 };
const PAL: FrameRate = { numerator: 25, denominator: 1 };

describe('time helpers beyond the shared vectors', () => {
  it('divides exactly near the end of the range', () => {
    const max = Number.MAX_SAFE_INTEGER;
    expect(divideRounded(max, 23_543_520, 'floor')).toBe(Number(BigInt(max) / 23_543_520n));
    expect(divideRounded(-max, 3, 'ceil')).toBe(Number(-BigInt(max) / 3n));
    expect(divideRounded(-7, 2, 'nearest')).toBe(-4);
    expect(divideRounded(7, 2, 'nearest')).toBe(4);
  });

  it('converts between pixels and flicks at a zoom level', () => {
    const twoSeconds = (2 * FLICKS_PER_SECOND) as Flicks;
    expect(flicksToPixels(twoSeconds, 100)).toBe(200);
    expect(pixelsToFlicks(200, 100)).toBe(twoSeconds);
    expect(pixelsToFlicks(0.4, 1_000_000)).toBe(282);
  });

  it('clamps UI-computed values into the flicks range', () => {
    expect(clampFlicks(1.5)).toBe(2);
    expect(clampFlicks(-1.5)).toBe(-2);
    expect(clampFlicks(Number.POSITIVE_INFINITY)).toBe(Number.MAX_SAFE_INTEGER);
    expect(clampFlicks(Number.NaN)).toBe(0);
  });

  it('snaps times to frame starts', () => {
    const frame = 23_543_520;
    expect(snapToFrame((frame + 1) as Flicks, NTSC, 'floor')).toBe(frame);
    expect(snapToFrame((frame * 1.6) as Flicks, NTSC, 'nearest')).toBe(frame * 2);
  });

  it('lands on the same frame after formatting then parsing', () => {
    for (const frame of [0, 1, 29, 30, 1799, 107_892]) {
      const time = (frame * 23_543_520) as Flicks;
      expect(parseTimecode(formatTimecode(time, NTSC), NTSC)).toEqual({ ok: true, value: time });
    }
  });

  it('never returns a negative zero', () => {
    const parsed = parseTimecode('-0s', PAL);
    expect(parsed.ok && Object.is(parsed.value, -0)).toBe(false);
  });

  it('shows frame rates like the core', () => {
    expect(formatFrameRate(PAL)).toBe('25');
    expect(formatFrameRate(NTSC)).toBe('29.97');
    expect(formatFrameRate({ numerator: 24000, denominator: 1001 })).toBe('23.976');
  });
});
