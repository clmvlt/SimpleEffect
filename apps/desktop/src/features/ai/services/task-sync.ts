import type { CoreClient, ProjectId, ProjectPatch, SequenceId, Task, TaskId } from '@se/protocol';

import {
  aiTasksStore,
  isTaskActive,
  itemTouchesScope,
  markUserEdited,
  pulseItems,
  resetAiTasks,
  setProjectTasks,
  upsertMirroredTask,
} from '../../../shared/ai-tasks';
import { listTasks } from '../api';

/** The parts of {@link CoreClient} the sync needs (a fake implements them in tests). */
export type TaskSyncSource = Pick<CoreClient, 'state' | 'onStateChange' | 'send' | 'on'>;

/** What the sync tells the app beyond the shared store. */
export interface TaskSyncEffects {
  /** A run of a task ended (it was queued or running, it is not any more). */
  readonly onRunFinished: (task: Task) => void;
}

/**
 * The running tasks whose scope a user patch touched: the agent may meet `REVISION_CONFLICT`
 * if it changes those items (an upserted item's sequence comes from its track, which every
 * patch refreshes with its items).
 */
export function tasksEditedByUser(patch: ProjectPatch, tasks: readonly Task[]): TaskId[] {
  if (patch.actor.kind !== 'user') return [];
  const sequenceOf = new Map<string, SequenceId>(
    patch.upserts.tracks.map((track) => [track.id, track.sequenceId]),
  );
  const running = tasks.filter((task) => task.status.kind === 'running');
  return running
    .filter((task) =>
      patch.upserts.items.some((item) => {
        const sequenceId = sequenceOf.get(item.trackId);
        return sequenceId !== undefined && itemTouchesScope(item, sequenceId, task.scope);
      }),
    )
    .map((task) => task.id);
}

/** Applies a patch of the project to the AI mirror: AI edits pulse, user edits are noted. */
export function applyPatchToTasks(patch: ProjectPatch): void {
  if (patch.actor.kind === 'aiTask') {
    pulseItems(patch.upserts.items.map((item) => item.id));
    return;
  }
  markUserEdited(tasksEditedByUser(patch, aiTasksStore.getState().tasks));
}

/**
 * Mirrors the tasks of `projectId` into the shared AI store: `listTasks` each time the connection
 * opens (a reconnection may reach a restarted core), then `taskUpdated` and `projectPatched`
 * events. Answers arriving after the connection changed again are dropped. Returns a function
 * that stops and forgets the mirror.
 */
export function connectTaskSync(
  client: TaskSyncSource,
  projectId: ProjectId,
  effects: TaskSyncEffects,
): () => void {
  let generation = 0;
  let stopped = false;
  const load = () => {
    generation += 1;
    const current = generation;
    if (client.state !== 'open') return;
    listTasks(client, projectId).then(
      (tasks) => {
        if (!stopped && current === generation) setProjectTasks(projectId, tasks);
      },
      (error: unknown) => {
        console.warn('[ai] listing tasks failed', error);
      },
    );
  };
  const unsubscribeState = client.onStateChange(load);
  const unsubscribeTasks = client.on('taskUpdated', ({ task }) => {
    if (stopped || task.projectId !== projectId) return;
    const previous = aiTasksStore.getState().tasks.find((known) => known.id === task.id);
    upsertMirroredTask(task);
    if (previous !== undefined && isTaskActive(previous) && !isTaskActive(task)) {
      effects.onRunFinished(task);
    }
  });
  const unsubscribePatches = client.on('projectPatched', (patch) => {
    if (!stopped && patch.projectId === projectId) applyPatchToTasks(patch);
  });
  setProjectTasks(projectId, []);
  if (client.state === 'open') load();
  return () => {
    stopped = true;
    unsubscribeState();
    unsubscribeTasks();
    unsubscribePatches();
    resetAiTasks();
  };
}
