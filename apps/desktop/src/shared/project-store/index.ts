/**
 * Read-only mirror of the open project (`docs/architecture.md` §5.6): a Zustand store fed by the
 * core's snapshot and `projectPatched` events. Features read it with `useProjectSelector` and
 * the selectors; they never write it (changes are commands to the core).
 */
export { applyPatch, mirrorFromSession } from './project-mirror';
export type { PatchOutcome, ProjectMirror } from './project-mirror';
export { createProjectStore } from './project-store';
export type { ProjectStore, ProjectStoreState } from './project-store';
export { connectProjectSync } from './project-sync';
export type { ProjectSync, ProjectSyncClient } from './project-sync';
export { ProjectStoreProvider } from './ProjectStoreProvider';
export type { ProjectStoreProviderProps } from './ProjectStoreProvider';
export {
  selectActiveSequence,
  selectClosedProjectId,
  selectHistory,
  selectProjectId,
  selectProjectName,
} from './selectors';
export { useProjectSelector, useProjectSync } from './use-project-store';
