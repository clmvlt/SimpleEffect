import { updatePeakHold, type PeakHoldState } from '@se/ui';
import { useEffect, useState } from 'react';

import type { PlaybackSession } from '../services/playback-session';

/** What the master meter shows, in dBFS: RMS bars and held sample peaks. */
export interface MeterReading {
  readonly levels: readonly number[];
  readonly peaks: readonly number[];
}

const SILENT: MeterReading = {
  levels: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
  peaks: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
};

/** Meter refreshes per second: smooth to the eye, cheap for React. */
const REFRESH_MS = 50;

/** Whether two readings look the same (half-decibel steps: silence never re-renders). */
function sameReading(left: MeterReading, right: MeterReading): boolean {
  const close = (a: readonly number[], b: readonly number[]) =>
    a.length === b.length &&
    a.every((value, index) => {
      const other = b[index] ?? Number.NEGATIVE_INFINITY;
      return value === other || Math.abs(value - other) < 0.5;
    });
  return close(left.levels, right.levels) && close(left.peaks, right.peaks);
}

/**
 * The master levels of `session`, polled 20 times per second with peak hold (`updatePeakHold`);
 * only the meter re-renders, and only when the reading changes.
 */
export function useMeterLevels(session: PlaybackSession): MeterReading {
  const [reading, setReading] = useState<MeterReading>(SILENT);
  useEffect(() => {
    let hold: PeakHoldState | undefined;
    const timer = setInterval(() => {
      const { peakDb, rmsDb } = session.levels();
      hold = updatePeakHold(hold, peakDb, performance.now());
      const next = { levels: rmsDb, peaks: hold.peaks };
      setReading((previous) => (sameReading(previous, next) ? previous : next));
    }, REFRESH_MS);
    return () => {
      clearInterval(timer);
    };
  }, [session]);
  return reading;
}
