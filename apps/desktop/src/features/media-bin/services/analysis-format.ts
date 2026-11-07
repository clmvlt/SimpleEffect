import type { AudioSummary, Flicks } from '@se/protocol';

import { flicksToSeconds } from '../../../shared/time';

/** `value` with one decimal in `locale` (`-16,5` in French). */
export function oneDecimal(value: number, locale: string): string {
  return new Intl.NumberFormat(locale, {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format(value);
}

/** A duration in seconds with one decimal (`2,3`), for totals of silences. */
export function seconds(duration: Flicks, locale: string): string {
  return oneDecimal(flicksToSeconds(duration), locale);
}

/** The analysis values the details panel shows, formatted; `null` for values not measured. */
export interface AnalysisFigures {
  readonly loudness: string | null;
  readonly truePeak: string | null;
  readonly loudnessRange: string | null;
  readonly silenceCount: number;
  readonly silenceSeconds: string;
}

/** The figures of an audio summary in `locale`. */
export function audioFigures(summary: AudioSummary, locale: string): AnalysisFigures {
  const decimal = (value: number | null) => (value === null ? null : oneDecimal(value, locale));
  return {
    loudness: decimal(summary.integratedLufs),
    truePeak: decimal(summary.truePeakDbtp),
    loudnessRange: decimal(summary.loudnessRangeLu),
    silenceCount: summary.silenceCount,
    silenceSeconds: seconds(summary.silenceDuration, locale),
  };
}
