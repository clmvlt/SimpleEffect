import { useEffect } from 'react';

import { uiPreferencesStorage, type KeyValueStorage } from '../../../shared/storage';
import { connectTimelinePersistence } from '../store/timeline-persistence';

/**
 * Restores the timeline view saved for `projectId` on this machine and saves its changes while
 * the project is shown (`timeline-persistence.ts`).
 */
export function useTimelinePersistence(
  projectId: string | null,
  storage: () => KeyValueStorage = uiPreferencesStorage,
): void {
  useEffect(() => {
    if (projectId === null) return undefined;
    return connectTimelinePersistence(projectId, storage());
  }, [projectId, storage]);
}
