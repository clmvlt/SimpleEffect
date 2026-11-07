import type { HistoryState, SequenceEntity } from '@se/protocol';

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

/**
 * The sequence the editor shows: the first one of the project.
 * TODO(prompt-16): choose the active sequence among several.
 */
export const selectActiveSequence = (state: ProjectStoreState): SequenceEntity | null => {
  const mirror = state.mirror;
  const firstId = mirror?.sequenceOrder[0];
  return firstId === undefined ? null : (mirror?.sequences[firstId] ?? null);
};

/** Id of a project the core closed while it was loaded, or `null`. */
export const selectClosedProjectId = (state: ProjectStoreState): string | null =>
  state.closedProjectId;
