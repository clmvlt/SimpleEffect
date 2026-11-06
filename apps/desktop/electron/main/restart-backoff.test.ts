import { describe, expect, it } from 'vitest';

import { restartDelayMs } from './restart-backoff';

describe('restartDelayMs', () => {
  it('doubles the delay after each consecutive failure', () => {
    expect([0, 1, 2, 3].map(restartDelayMs)).toEqual([250, 500, 1000, 2000]);
  });

  it('is capped at 10 seconds', () => {
    expect(restartDelayMs(6)).toBe(10_000);
    expect(restartDelayMs(1000)).toBe(10_000);
  });

  it('treats invalid attempt counts as a first failure', () => {
    expect(restartDelayMs(-3)).toBe(250);
  });
});
