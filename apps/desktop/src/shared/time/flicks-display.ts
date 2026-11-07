import type { Flicks } from '@se/protocol';

/**
 * Display helpers for flicks (705,600,000 per second, ADR-0004). Floats appear only here, at the
 * UI boundary. TODO(prompt-10): the timeline adds the timecode helpers checked against
 * `docs/test-vectors/*.json`; keep these display-only.
 */

/** Flicks in one second. */
export const FLICKS_PER_SECOND = 705_600_000;

/** Seconds of a flicks value (display only, never for document positions). */
export function flicksToSeconds(flicks: Flicks): number {
  return flicks / FLICKS_PER_SECOND;
}

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
