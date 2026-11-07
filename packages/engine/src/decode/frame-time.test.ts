import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { NTSC_30, frameTime, seconds } from '../testing/document';
import { frameLookupTime } from './frame-time';

describe('frameLookupTime', () => {
  it('aims at the middle of the frame shown, computed exactly in flicks', () => {
    expect(frameLookupTime(frameTime(31, NTSC_30), NTSC_30)).toBeCloseTo((31.5 * 1001) / 30000, 12);
    expect(frameLookupTime((frameTime(31, NTSC_30) - 1) as Flicks, NTSC_30)).toBeCloseTo(
      (30.5 * 1001) / 30000,
      12,
    );
  });

  it('stays inside a frame stored with millisecond timestamps', () => {
    // Frame 5 at 29.97 fps starts at 166.83 ms; Matroska stores 167 ms.
    const lookup = frameLookupTime(frameTime(5, NTSC_30), NTSC_30);
    expect(lookup).toBeGreaterThan(0.167);
    expect(lookup).toBeLessThan(0.2);
  });

  it('falls back to a microsecond past the time without a frame rate', () => {
    expect(frameLookupTime(seconds(1.5), null)).toBeCloseTo(1.500001, 9);
  });
});
