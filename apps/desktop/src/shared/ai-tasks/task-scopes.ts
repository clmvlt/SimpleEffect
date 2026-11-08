import type { ItemEntity, SequenceId, Task, TaskId, TaskScope, TimeRange } from '@se/protocol';

/**
 * Pure facts about AI tasks shared by the AI feature and the timeline: their numbers (« IA #3 »),
 * which ones are active, and whether an item lies inside a task's scope.
 */

/** Whether a task waits or runs. */
export function isTaskActive(task: Task): boolean {
  return task.status.kind === 'queued' || task.status.kind === 'running';
}

/**
 * Numbers of the project's tasks, from 1 for the oldest (`tasks` come oldest first from the core,
 * and tasks are never deleted, so a number never changes).
 */
export function taskNumbers(tasks: readonly Task[]): ReadonlyMap<TaskId, number> {
  const ordered = [...tasks].sort(
    (left, right) => left.createdAt - right.createdAt || left.id.localeCompare(right.id),
  );
  return new Map(ordered.map((task, index) => [task.id, index + 1]));
}

/** End of a time range (exclusive). */
export function rangeEnd(range: TimeRange): number {
  return range.start + range.duration;
}

/**
 * Whether `item` (of `sequenceId`) touches `scope`: same sequence, on one of its tracks, and
 * overlapping its range. Overlap rather than containment: an edit there may meet the agent's.
 */
export function itemTouchesScope(
  item: ItemEntity,
  sequenceId: SequenceId,
  scope: TaskScope,
): boolean {
  if (sequenceId !== scope.sequenceId) return false;
  if (scope.tracks !== null && !scope.tracks.includes(item.trackId)) return false;
  if (scope.range === null) return true;
  const start = item.timelineRange.start;
  const end = start + item.timelineRange.duration;
  return start < rangeEnd(scope.range) && end > scope.range.start;
}

/** The active tasks working on `sequenceId`, oldest first. */
export function activeTasksOn(tasks: readonly Task[], sequenceId: SequenceId): Task[] {
  return tasks.filter((task) => isTaskActive(task) && task.scope.sequenceId === sequenceId);
}
