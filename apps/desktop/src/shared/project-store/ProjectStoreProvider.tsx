import type { ReactNode } from 'react';

import type { ProjectStore } from './project-store';
import { ProjectStoreContext } from './project-store-context';
import type { ProjectSync } from './project-sync';

/** Props of {@link ProjectStoreProvider}. */
export interface ProjectStoreProviderProps {
  readonly store: ProjectStore;
  readonly sync: ProjectSync;
  readonly children: ReactNode;
}

/** Makes the project store readable below it (`useProjectSelector`). */
export function ProjectStoreProvider({ store, sync, children }: ProjectStoreProviderProps) {
  return <ProjectStoreContext value={{ store, sync }}>{children}</ProjectStoreContext>;
}
