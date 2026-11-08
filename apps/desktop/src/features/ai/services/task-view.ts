import type { Task, TaskUsage } from '@se/protocol';

import { isTaskActive } from '../../../shared/ai-tasks';

/**
 * Pure helpers of the tasks panel: the order of the cards, elapsed times and the usage of the
 * project's tasks.
 */

const STATUS_RANK = { running: 0, queued: 1 } as const;

/**
 * Running tasks first (oldest first), then queued ones (in submission order), then finished ones,
 * most recently changed first.
 */
export function orderedTasks(tasks: readonly Task[]): Task[] {
  const active = tasks
    .filter(isTaskActive)
    .sort(
      (left, right) =>
        STATUS_RANK[left.status.kind as keyof typeof STATUS_RANK] -
          STATUS_RANK[right.status.kind as keyof typeof STATUS_RANK] ||
        left.createdAt - right.createdAt,
    );
  const finished = tasks
    .filter((task) => !isTaskActive(task))
    .sort((left, right) => right.updatedAt - left.updatedAt);
  return [...active, ...finished];
}

/**
 * Milliseconds the latest run has taken: until `now` while it runs, until it finished otherwise;
 * `null` before it started.
 */
export function elapsedMs(task: Task, now: number): number | null {
  const run = task.runs.at(-1);
  if (run?.startedAt == null) return null;
  const end = run.finishedAt ?? (task.status.kind === 'running' ? now : run.startedAt);
  return Math.max(0, end - run.startedAt);
}

/** The usage of every task added up (costs of the tasks that report one). */
export function totalUsage(tasks: readonly Task[]): TaskUsage {
  return tasks.reduce<TaskUsage>(
    (total, { usage }) => ({
      inputTokens: total.inputTokens + usage.inputTokens,
      outputTokens: total.outputTokens + usage.outputTokens,
      costUsd: usage.costUsd === null ? total.costUsd : (total.costUsd ?? 0) + usage.costUsd,
    }),
    { inputTokens: 0, outputTokens: 0, costUsd: null },
  );
}

/** Whether a usage reports anything. */
export function hasUsage(usage: TaskUsage): boolean {
  return usage.inputTokens + usage.outputTokens > 0 || usage.costUsd !== null;
}
