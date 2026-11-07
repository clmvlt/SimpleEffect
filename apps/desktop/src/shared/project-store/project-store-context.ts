import { createContext } from 'react';

import type { ProjectStore } from './project-store';
import type { ProjectSync } from './project-sync';

/** The project store and its sync, provided once by `app/`. */
export interface ProjectStoreContextValue {
  readonly store: ProjectStore;
  readonly sync: ProjectSync;
}

/** Context of {@link ProjectStoreContextValue}; `null` outside `ProjectStoreProvider`. */
export const ProjectStoreContext = createContext<ProjectStoreContextValue | null>(null);
