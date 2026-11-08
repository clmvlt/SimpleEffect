/**
 * The AI tasks of the open project as features see them (prompt 20): a read-only mirror fed by
 * the AI feature's task sync (`setProjectTasks`, `upsertMirroredTask`), task numbers, pulses of
 * items an AI patch changed, running tasks whose scope the user edited, « Voir » requests
 * (`showTask`) and composer requests (`openComposer`). The timeline reads it for its overlays.
 */
export {
  aiTasksStore,
  markUserEdited,
  PULSE_DURATION_MS,
  pulseItems,
  resetAiTasks,
  setProjectTasks,
  showTask,
  upsertMirroredTask,
  useAiTasks,
  useItemPulse,
  useTaskNumber,
} from './ai-tasks-store';
export type { AiTasksState } from './ai-tasks-store';
export { composerRequestStore, openComposer, useComposerRequest } from './composer-requests';
export type { ComposerRequest } from './composer-requests';
export {
  activeTasksOn,
  isTaskActive,
  itemTouchesScope,
  rangeEnd,
  taskNumbers,
} from './task-scopes';
