import type { HistoryState, ProjectRecovery, ProjectSaveState } from '@se/protocol';

import type { ProjectStoreState } from './project-store';

/**
 * Typed selectors of the project store. They return references held by the store (never new
 * arrays or objects), so components re-render only when what they show changed.
 */

/** Id of the loaded project, or `null`. */
export const selectProjectId = (state: ProjectStoreState): string | null =>
  state.mirror?.projectId ?? null;

/** Name of the loaded project, or `null`. */
export const selectProjectName = (state: ProjectStoreState): string | null =>
  state.mirror?.project.name ?? null;

/** Undo state of the loaded project, or `null`. */
export const selectHistory = (state: ProjectStoreState): HistoryState | null =>
  state.mirror?.history ?? null;

/** Id of a project the core closed while it was loaded, or `null`. */
export const selectClosedProjectId = (state: ProjectStoreState): string | null =>
  state.closedProjectId;

/** What opening the loaded project repaired, or `null` (see `ProjectMirror.recovery`). */
export const selectRecovery = (state: ProjectStoreState): ProjectRecovery | null =>
  state.mirror?.recovery ?? null;

/** The loaded project's latest reported save state, or `null` while saving works. */
export const selectSaveState = (state: ProjectStoreState): ProjectSaveState | null =>
  state.saveState;
