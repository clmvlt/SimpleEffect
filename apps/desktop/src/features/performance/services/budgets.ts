import type { PerformanceReport } from './measure-performance';

/**
 * The performance budgets of `docs/performance.md` (prompt 29) checked by the in-app measure.
 * Change both together.
 */
export const BUDGETS = {
  /** Round trip of a command to se-core over the local socket, 95th percentile. */
  coreLatencyMs: 20,
  /** A scrub step: playhead moved → frame drawn, 95th percentile. */
  scrubMs: 100,
  /** Main-thread frame interval while the viewer works, 95th percentile (≥ 50 fps). */
  frameIntervalMs: 20,
  /** A frame interval longer than this is a visible hitch. */
  longFrameMs: 50,
  /** Frames dropped while playing a few seconds at 1×. */
  droppedFrames: 0,
} as const;

/** One line of the result table. */
export interface BudgetRow {
  readonly id: 'coreLatency' | 'scrub' | 'frameInterval' | 'longFrames' | 'droppedFrames';
  /** The measured value (milliseconds or a count), `null` when it could not be measured. */
  readonly value: number | null;
  readonly budget: number;
  readonly unit: 'ms' | 'count';
  readonly status: 'within' | 'over' | 'skipped';
}

function row(
  id: BudgetRow['id'],
  value: number | null,
  budget: number,
  unit: BudgetRow['unit'],
): BudgetRow {
  const status = value === null ? 'skipped' : value <= budget ? 'within' : 'over';
  return { id, value, budget, unit, status };
}

/** The report checked against {@link BUDGETS}, in the order the panel shows them. */
export function budgetRows(report: PerformanceReport): BudgetRow[] {
  return [
    row('coreLatency', report.core.p95, BUDGETS.coreLatencyMs, 'ms'),
    row('scrub', report.scrub?.p95 ?? null, BUDGETS.scrubMs, 'ms'),
    row('frameInterval', report.frames?.p95 ?? null, BUDGETS.frameIntervalMs, 'ms'),
    row('longFrames', report.longFrames, 0, 'count'),
    row('droppedFrames', report.playback?.droppedFrames ?? null, BUDGETS.droppedFrames, 'count'),
  ];
}
