import type { ProjectId, ProjectSaveState } from '@se/protocol';
import { createStore, type StoreApi } from 'zustand/vanilla';

import type { ProjectMirror } from './project-mirror';

/** State of the project store. */
export interface ProjectStoreState {
  /** The project shown by the editor, or `null` when none is loaded. */
  readonly mirror: ProjectMirror | null;
  /** Set when the core closed the loaded project (it is no longer editable). */
  readonly closedProjectId: ProjectId | null;
  /**
   * The latest save state the core reported for the loaded project (`projectSaveStateChanged`),
   * or `null` while none was reported since it was loaded (saving works).
   */
  readonly saveState: ProjectSaveState | null;
}

/**
 * The store holding the mirror of the open project. Only `project-sync.ts` writes it; features
 * read it through the hooks of this module (`useProjectSelector`).
 */
export type ProjectStore = StoreApi<ProjectStoreState>;

/** An empty project store. */
export function createProjectStore(): ProjectStore {
  return createStore<ProjectStoreState>()(() => ({
    mirror: null,
    closedProjectId: null,
    saveState: null,
  }));
}
