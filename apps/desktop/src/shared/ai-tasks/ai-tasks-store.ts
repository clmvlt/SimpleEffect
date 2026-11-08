import type { ItemId, ProjectId, Task, TaskId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { showSidePanelTab } from '../side-panel';
import { taskNumbers } from './task-scopes';

/**
 * The AI tasks of the open project as the renderer mirrors them (fed by the AI feature's task
 * sync, read by the AI panel and the timeline's overlays), plus short-lived UI facts: items an AI
 * patch just changed (they pulse), running tasks whose scope the user edited meanwhile, and the
 * task a « Voir » asked to show.
 */
export interface AiTasksState {
  readonly projectId: ProjectId | null;
  /** Oldest first. */
  readonly tasks: readonly Task[];
  readonly numbers: ReadonlyMap<TaskId, number>;
  /** Active tasks whose scope the user edited since they started (they may meet a conflict). */
  readonly userEdited: ReadonlySet<TaskId>;
  /** Items an AI patch just changed → a token that changes with every new pulse. */
  readonly pulses: ReadonlyMap<ItemId, number>;
  /** The task to show in the AI panel; `nonce` changes with every request. */
  readonly focus: { readonly taskId: TaskId; readonly nonce: number } | null;
}

const EMPTY: AiTasksState = {
  projectId: null,
  tasks: [],
  numbers: new Map(),
  userEdited: new Set(),
  pulses: new Map(),
  focus: null,
};

/** How long an item pulses after an AI patch changed it. */
export const PULSE_DURATION_MS = 1200;

/** The store (vanilla, so services and tests use it outside React). */
export const aiTasksStore = createStore<AiTasksState>()(() => EMPTY);

let pulseToken = 0;
let focusNonce = 0;

/** Replaces the tasks of `projectId` (a new project or a reload of the list). */
export function setProjectTasks(projectId: ProjectId, tasks: readonly Task[]): void {
  const state = aiTasksStore.getState();
  const sameProject = state.projectId === projectId;
  aiTasksStore.setState({
    ...(sameProject ? state : EMPTY),
    projectId,
    tasks,
    numbers: taskNumbers(tasks),
  });
}

/** Puts `task` in place (or appends it) when it belongs to the mirrored project. */
export function upsertMirroredTask(task: Task): void {
  const state = aiTasksStore.getState();
  if (state.projectId !== task.projectId) return;
  const index = state.tasks.findIndex((candidate) => candidate.id === task.id);
  const tasks =
    index === -1
      ? [...state.tasks, task]
      : state.tasks.map((candidate, at) => (at === index ? task : candidate));
  const active = task.status.kind === 'queued' || task.status.kind === 'running';
  const userEdited = new Set(state.userEdited);
  if (!active) userEdited.delete(task.id);
  aiTasksStore.setState({ tasks, numbers: taskNumbers(tasks), userEdited });
}

/** Records that the user edited inside the scope of these active tasks. */
export function markUserEdited(taskIds: readonly TaskId[]): void {
  const { userEdited } = aiTasksStore.getState();
  if (taskIds.every((id) => userEdited.has(id))) return;
  aiTasksStore.setState({ userEdited: new Set([...userEdited, ...taskIds]) });
}

/** Makes `itemIds` pulse for {@link PULSE_DURATION_MS}. */
export function pulseItems(itemIds: readonly ItemId[]): void {
  if (itemIds.length === 0) return;
  pulseToken += 1;
  const token = pulseToken;
  const pulses = new Map(aiTasksStore.getState().pulses);
  for (const id of itemIds) pulses.set(id, token);
  aiTasksStore.setState({ pulses });
  setTimeout(() => {
    const current = new Map(aiTasksStore.getState().pulses);
    let changed = false;
    for (const id of itemIds) {
      if (current.get(id) === token) {
        current.delete(id);
        changed = true;
      }
    }
    if (changed) aiTasksStore.setState({ pulses: current });
  }, PULSE_DURATION_MS);
}

/** Shows the AI panel scrolled to `taskId` (the « Voir » of badges, toasts and notifications). */
export function showTask(taskId: TaskId): void {
  focusNonce += 1;
  aiTasksStore.setState({ focus: { taskId, nonce: focusNonce } });
  showSidePanelTab('ai');
}

/** Forgets everything (the project closed; tests). */
export function resetAiTasks(): void {
  aiTasksStore.setState(EMPTY);
}

/** Reads the store with `selector` (return stored references, never new arrays). */
export function useAiTasks<T>(selector: (state: AiTasksState) => T): T {
  return useStore(aiTasksStore, selector);
}

/** The number of `taskId` (« #3 »), when it is a task of the open project. */
export function useTaskNumber(taskId: TaskId | null): number | undefined {
  return useAiTasks((state) => (taskId === null ? undefined : state.numbers.get(taskId)));
}

/** The token of `itemId`'s current pulse, if it pulses. */
export function useItemPulse(itemId: ItemId): number | undefined {
  return useAiTasks((state) => state.pulses.get(itemId));
}
