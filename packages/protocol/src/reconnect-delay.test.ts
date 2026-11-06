import { describe, expect, it } from 'vitest';

import { reconnectDelayMs } from './reconnect-delay';

describe('reconnectDelayMs', () => {
  it('doubles from 250 ms and caps at 5 s by default', () => {
    expect([0, 1, 2, 3, 4, 5, 6].map((failures) => reconnectDelayMs(failures))).toEqual([
      250, 500, 1_000, 2_000, 4_000, 5_000, 5_000,
    ]);
  });

  it('honours custom bounds', () => {
    expect(reconnectDelayMs(3, { initialMs: 10, maxMs: 50 })).toBe(50);
    expect(reconnectDelayMs(-1, { initialMs: 10, maxMs: 50 })).toBe(10);
  });
});
