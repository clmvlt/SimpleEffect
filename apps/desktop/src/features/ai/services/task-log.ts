import type { Task, TaskEventRecord, TaskRevertReport } from '@se/protocol';

import { isTaskActive } from '../../../shared/ai-tasks';

/** `events` with `record` added in sequence order (a duplicate sequence is ignored). */
export function appendEvent(
  events: readonly TaskEventRecord[],
  record: TaskEventRecord,
): TaskEventRecord[] {
  if (events.some((event) => event.sequence === record.sequence)) return [...events];
  return [...events, record].sort((left, right) => left.sequence - right.sequence);
}

/** What a user can do with a task now. */
export interface TaskActions {
  readonly cancel: boolean;
  readonly retry: boolean;
  readonly followUp: boolean;
  readonly revert: boolean;
  /** « Voir les modifications »: select what it changed. */
  readonly changes: boolean;
}

/**
 * Cancel while active; retry once failed or cancelled; follow up once its session is known (and
 * the provider can resume sessions); revert and show the changes once finished with edits (an
 * open changeset).
 */
export function taskActions(task: Task, resumeSessions = true): TaskActions {
  const active = isTaskActive(task);
  const edited = !active && task.changesetId !== null;
  return {
    cancel: active,
    retry: task.status.kind === 'failed' || task.status.kind === 'cancelled',
    followUp: !active && resumeSessions && task.agentSessionId !== null,
    revert: edited,
    changes: edited,
  };
}

/** How a revert went, for its toast: everything undone, some edits kept, or nothing undone. */
export type RevertOutcome =
  | { readonly kind: 'reverted' }
  | { readonly kind: 'partial'; readonly kept: number }
  | { readonly kind: 'nothing'; readonly kept: number };

/** The outcome of a revert report. */
export function revertOutcome(report: TaskRevertReport): RevertOutcome {
  const kept = report.conflicts.length;
  if (!report.reverted) return { kind: 'nothing', kept };
  return kept === 0 ? { kind: 'reverted' } : { kind: 'partial', kept };
}
