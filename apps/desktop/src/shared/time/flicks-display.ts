import type { Flicks } from '@se/protocol';

import { flicksToSeconds } from './flicks';

/** Display helpers for durations (media bin). Timecodes are in `timecode.ts`. */

/** A duration as `M:SS` or `H:MM:SS` (whole seconds, rounded down), e.g. `1:05:09`. */
export function formatDuration(flicks: Flicks): string {
  const total = Math.max(0, Math.floor(flicksToSeconds(flicks)));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, '0');
  return hours > 0
    ? `${String(hours)}:${String(minutes).padStart(2, '0')}:${seconds}`
    : `${String(minutes)}:${seconds}`;
}
