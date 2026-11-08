import type { TaskUsage } from '@se/protocol';

/** Tokens and cost of a usage, formatted for `locale` (`12,4 k`, `0,02 $US`). */
export interface UsageText {
  readonly tokens: string;
  /** `null` when no run reported a cost. */
  readonly cost: string | null;
}

/** The usage as short localised numbers. */
export function usageText(usage: TaskUsage, locale: string): UsageText {
  const tokens = new Intl.NumberFormat(locale, {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(usage.inputTokens + usage.outputTokens);
  const cost =
    usage.costUsd === null
      ? null
      : new Intl.NumberFormat(locale, {
          style: 'currency',
          currency: 'USD',
          minimumFractionDigits: 2,
          maximumFractionDigits: usage.costUsd < 0.1 ? 3 : 2,
        }).format(usage.costUsd);
  return { tokens, cost };
}
