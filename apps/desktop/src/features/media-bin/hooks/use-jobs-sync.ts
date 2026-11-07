import { useEffect } from 'react';

import { useConnectionStatus, useCoreClient } from '../../../shared/core';
import { connectJobsSync } from './jobs-sync';

/** Keeps the jobs store current for the open project (reloaded after every reconnection). */
export function useJobsSync(projectId: string | null): void {
  const client = useCoreClient();
  const open = useConnectionStatus() === 'open';
  useEffect(() => {
    if (projectId === null || !open) {
      return undefined;
    }
    return connectJobsSync(client, projectId);
  }, [client, projectId, open]);
}
