import type { RecentProject } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

import { listRecentProjects, type ProjectsClient } from '../api';

/** UI state of the home screen's recent list. */
export interface RecentProjectsState {
  readonly projects: readonly RecentProject[];
  /** `loading` until the first answer; `failed` if the last refresh failed. */
  readonly status: 'loading' | 'ready' | 'failed';
  /** Text typed in the search field. */
  readonly query: string;
}

/** The recent projects store of the home screen. */
export const recentProjectsStore = createStore<RecentProjectsState>()(() => ({
  projects: [],
  status: 'loading',
  query: '',
}));

/** Reloads the list from the core. */
export async function refreshRecentProjects(client: ProjectsClient): Promise<void> {
  try {
    const projects = await listRecentProjects(client);
    recentProjectsStore.setState({ projects, status: 'ready' });
  } catch {
    recentProjectsStore.setState({ status: 'failed' });
  }
}

/** Changes the search text. */
export function setRecentProjectsQuery(query: string): void {
  recentProjectsStore.setState({ query });
}
