/** Level zones of a meter, in dBFS: green below `warningDb`, orange up to `dangerDb`, red above. */
export interface MeterZones {
  readonly warningDb: number;
  readonly dangerDb: number;
}

/** Default zones: orange from −12 dBFS, red from −3 dBFS. */
export const DEFAULT_METER_ZONES: MeterZones = { warningDb: -12, dangerDb: -3 };

/**
 * Position of a level on a meter scale, from 0 (at or below `minDb`, silence included) to 1 (at or
 * above `maxDb`). Linear in dB, like most editor meters.
 */
export function levelToFraction(db: number, minDb: number, maxDb: number): number {
  if (Number.isNaN(db) || db <= minDb) {
    return 0;
  }
  return Math.min(1, (db - minDb) / (maxDb - minDb));
}

/** Peak-hold memory of a meter, one entry per channel. */
export interface PeakHoldState {
  readonly peaks: readonly number[];
  /** When each peak was last raised, in milliseconds (same clock as `nowMs`). */
  readonly raisedAt: readonly number[];
  readonly updatedAt: number;
}

/** Peak-hold behaviour. */
export interface PeakHoldOptions {
  /** How long a peak stays still before falling (default 1500 ms). */
  readonly holdMs?: number | undefined;
  /** Fall speed after the hold (default 20 dB per second). */
  readonly decayDbPerSecond?: number | undefined;
}

/**
 * Next peak-hold state for new `levels` at `nowMs`: a peak jumps up immediately, stays for
 * `holdMs`, then falls at `decayDbPerSecond` (never below the current level). Pure, so the audio
 * code calls it at its own rate and passes `peaks` to `Meter`.
 */
export function updatePeakHold(
  previous: PeakHoldState | undefined,
  levels: readonly number[],
  nowMs: number,
  { holdMs = 1500, decayDbPerSecond = 20 }: PeakHoldOptions = {},
): PeakHoldState {
  const elapsedSeconds = previous === undefined ? 0 : (nowMs - previous.updatedAt) / 1000;
  const peaks: number[] = [];
  const raisedAt: number[] = [];
  levels.forEach((level, channel) => {
    const peak = previous?.peaks[channel] ?? Number.NEGATIVE_INFINITY;
    const peakTime = previous?.raisedAt[channel] ?? nowMs;
    if (level >= peak) {
      peaks.push(level);
      raisedAt.push(nowMs);
    } else if (nowMs - peakTime < holdMs) {
      peaks.push(peak);
      raisedAt.push(peakTime);
    } else {
      peaks.push(Math.max(level, peak - decayDbPerSecond * elapsedSeconds));
      raisedAt.push(peakTime);
    }
  });
  return { peaks, raisedAt, updatedAt: nowMs };
}
