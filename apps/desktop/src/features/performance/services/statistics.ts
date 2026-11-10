/** Distribution of a measured duration, in milliseconds. */
export interface DurationSummary {
  readonly count: number;
  readonly median: number;
  readonly p95: number;
  readonly max: number;
}

/** The `fraction` quantile of sorted `values` (nearest rank; 0 for none). */
function quantile(sorted: readonly number[], fraction: number): number {
  if (sorted.length === 0) return 0;
  const rank = Math.min(sorted.length - 1, Math.ceil(fraction * sorted.length) - 1);
  return sorted[Math.max(0, rank)] ?? 0;
}

/** Median, 95th percentile and maximum of `values` (milliseconds). */
export function summarize(values: readonly number[]): DurationSummary {
  const sorted = [...values].sort((a, b) => a - b);
  return {
    count: sorted.length,
    median: quantile(sorted, 0.5),
    p95: quantile(sorted, 0.95),
    max: sorted.at(-1) ?? 0,
  };
}
