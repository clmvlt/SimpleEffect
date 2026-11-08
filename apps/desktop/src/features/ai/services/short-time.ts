import type { Flicks, TimeRange } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../../../shared/time';

/**
 * Compact times for chips and task logs: `00:51`, `01:02.5`, `1:02:03` (hours only when needed,
 * at most two decimals, trailing zeros dropped). Language-neutral, like timecodes.
 */
export function shortTime(seconds: number): string {
  const negative = seconds < 0;
  const hundredths = Math.round(Math.abs(seconds) * 100);
  const whole = Math.floor(hundredths / 100);
  const fraction = hundredths % 100;
  const pad = (value: number) => String(value).padStart(2, '0');
  const hours = Math.floor(whole / 3600);
  const minutes = Math.floor(whole / 60) % 60;
  const clock =
    hours > 0
      ? `${String(hours)}:${pad(minutes)}:${pad(whole % 60)}`
      : `${pad(minutes)}:${pad(whole % 60)}`;
  const decimals = fraction === 0 ? '' : '.' + pad(fraction).replace(/0$/u, '');
  return (negative ? '-' : '') + clock + decimals;
}

/** {@link shortTime} of a flicks position. */
export function shortFlicks(time: Flicks): string {
  return shortTime(time / FLICKS_PER_SECOND);
}

/** `00:51 → 00:54`. */
export function shortRange(range: TimeRange): string {
  return `${shortFlicks(range.start)} → ${shortFlicks((range.start + range.duration) as Flicks)}`;
}
