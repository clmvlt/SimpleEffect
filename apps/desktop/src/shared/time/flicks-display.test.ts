import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { FLICKS_PER_SECOND, flicksToSeconds } from './flicks';
import { formatDuration } from './flicks-display';

const seconds = (value: number) => (value * FLICKS_PER_SECOND) as Flicks;

describe('flicks display', () => {
  it('converts to seconds', () => {
    expect(flicksToSeconds(seconds(2.5))).toBe(2.5);
  });

  it('formats durations with minutes, then hours', () => {
    expect(formatDuration(seconds(0))).toBe('0:00');
    expect(formatDuration(seconds(65.9))).toBe('1:05');
    expect(formatDuration(seconds(3909))).toBe('1:05:09');
  });
});
