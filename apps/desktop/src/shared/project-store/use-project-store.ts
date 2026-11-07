import { use } from 'react';
import { useStore } from 'zustand';

import type { ProjectMirror } from './project-mirror';
import type { ProjectStoreState } from './project-store';
import { ProjectStoreContext, type ProjectStoreContextValue } from './project-store-context';
import type { ProjectSync } from './project-sync';

function useProjectStoreContext(): ProjectStoreContextValue {
  const value = use(ProjectStoreContext);
  if (value === null) {
    throw new Error('Project store hooks must be used inside <ProjectStoreProvider>.');
  }
  return value;
}

/**
 * Reads the mirror of the open project. Read-only: change the project by sending a command to
 * the core. Pass a selector of `selectors.ts` (or one returning a stored reference).
 */
export function useProjectSelector<T>(selector: (state: ProjectStoreState) => T): T {
  return useStore(useProjectStoreContext().store, selector);
}

/**
 * A function reading the mirror as it is now (`null` without a project), for code that runs
 * after a command resolved (its patch has arrived by then). Read-only, like the selectors.
 */
export function useProjectMirrorReader(): () => ProjectMirror | null {
  const { store } = useProjectStoreContext();
  return () => store.getState().mirror;
}

/** Loads and forgets the editor's project (for `app/editor` only). */
export function useProjectSync(): ProjectSync {
  return useProjectStoreContext().sync;
}
