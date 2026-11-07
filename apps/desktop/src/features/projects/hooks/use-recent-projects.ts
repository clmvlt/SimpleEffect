import { useEffect } from 'react';
import { useStore } from 'zustand';

import { useConnectionStatus, useCoreClient } from '../../../shared/core';
import { filterProjects } from '../services/filter-projects';
import {
  recentProjectsStore,
  refreshRecentProjects,
  setRecentProjectsQuery,
} from '../store/recent-projects-store';

/**
 * The recent list of the home screen, reloaded whenever the connection to the core opens (at
 * start and after a core restart).
 */
export function useRecentProjects() {
  const client = useCoreClient();
  const connection = useConnectionStatus();
  const state = useStore(recentProjectsStore);
  useEffect(() => {
    if (connection === 'open') {
      void refreshRecentProjects(client);
    }
  }, [client, connection]);
  return {
    ...state,
    visible: filterProjects(state.projects, state.query),
    setQuery: setRecentProjectsQuery,
    refresh: () => refreshRecentProjects(client),
  };
}
