import { useEffect, useEffectEvent, useState } from 'react';

import { useCoreClient } from '../../shared/core';
import { errorMessageKey, type ErrorMessageKey } from '../../shared/errors';
import {
  selectClosedProjectId,
  selectProjectId,
  useProjectSelector,
  useProjectSync,
} from '../../shared/project-store';
import { loadEditorProject } from './load-editor-project';

/** Where the editor stands with the project of its URL. */
export type EditorProjectStatus =
  | { readonly kind: 'loading' }
  | { readonly kind: 'ready' }
  | { readonly kind: 'failed'; readonly errorKey: ErrorMessageKey }
  | { readonly kind: 'closed' };

/** Loads the project `projectId` into the project store unless it is already there. */
export function useEditorProject(projectId: string): EditorProjectStatus {
  const client = useCoreClient();
  const sync = useProjectSync();
  const loadedId = useProjectSelector(selectProjectId);
  const closedId = useProjectSelector(selectClosedProjectId);
  // Keyed by project so a failure never outlives the URL it belongs to.
  const [failure, setFailure] = useState<{ projectId: string; key: ErrorMessageKey } | null>(null);
  const alreadyLoaded = useEffectEvent(() => loadedId === projectId);

  useEffect(() => {
    if (alreadyLoaded()) return undefined;
    let current = true;
    loadEditorProject(client, projectId).then(
      (session) => {
        if (current) sync.load(session);
      },
      (error: unknown) => {
        if (current) setFailure({ projectId, key: errorMessageKey(error) });
      },
    );
    return () => {
      current = false;
    };
  }, [client, sync, projectId]);

  if (closedId === projectId) return { kind: 'closed' };
  if (loadedId === projectId) return { kind: 'ready' };
  if (failure?.projectId === projectId) return { kind: 'failed', errorKey: failure.key };
  return { kind: 'loading' };
}
