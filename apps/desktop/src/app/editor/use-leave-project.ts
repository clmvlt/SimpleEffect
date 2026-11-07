import { useNavigate } from 'react-router';

import { useCoreClient } from '../../shared/core';
import { useProjectSync } from '../../shared/project-store';
import { HOME_PATH } from '../../shared/routing';

/** Closes the editor's project (the core saves it) and goes back to the home screen. */
export function useLeaveProject(projectId: string): () => void {
  const client = useCoreClient();
  const sync = useProjectSync();
  const navigate = useNavigate();
  return () => {
    // Forget first, so the `projectClosed` event that follows is not shown as « Projet fermé ».
    sync.forget();
    client.send({ kind: 'closeProject', projectId }).catch((error: unknown) => {
      // Already closed (or the core restarted): nothing left to save.
      console.warn('[editor] closing the project failed', error);
    });
    void navigate(HOME_PATH);
  };
}
