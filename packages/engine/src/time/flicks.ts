import type { Flicks } from '@se/protocol';

/** Flicks in one second (ADR-0004): exact for every common frame and sample rate. */
export const FLICKS_PER_SECOND = 705_600_000;

/** `value` as flicks, rounded to the nearest flick (halves away from zero, like `se_domain`). */
export function roundToFlicks(value: number): Flicks {
  return (Math.sign(value) * Math.round(Math.abs(value))) as Flicks;
}
