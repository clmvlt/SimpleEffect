import type { Asset } from '@se/protocol';
import { useEffect, useRef } from 'react';

import { useCoreClient } from '../../../shared/core';
import { needsDecodabilityCheck } from '../services/asset-list';
import { reportDecodabilityOf } from './decodability-reporter';

/**
 * Checks every asset whose decodability is unknown (new imports, relinked content, undone
 * removals) once per content, and reports it to the core; a failed report is retried when the
 * assets change next.
 */
export function useDecodabilityReporter(projectId: string | null, assets: readonly Asset[]): void {
  const client = useCoreClient();
  const started = useRef(new Set<string>());
  useEffect(() => {
    if (projectId === null) {
      return;
    }
    for (const asset of assets) {
      const key = `${asset.id}:${asset.fingerprint?.sampleHash ?? ''}`;
      if (!needsDecodabilityCheck(asset) || started.current.has(key)) continue;
      started.current.add(key);
      reportDecodabilityOf(client, projectId, asset).catch((error: unknown) => {
        console.warn('[media-bin] reporting decodability failed', error);
        started.current.delete(key);
      });
    }
  }, [client, projectId, assets]);
}
