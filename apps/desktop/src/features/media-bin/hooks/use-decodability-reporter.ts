import type { Asset } from '@se/protocol';
import { useEffect, useRef } from 'react';

import { useCoreClient } from '../../../shared/core';
import { needsDecodabilityCheck } from '../services/asset-list';
import { reportCapabilitiesOf, reportDecodabilityOf } from './decodability-reporter';

/**
 * Reports what this machine decodes among the codecs of mezzanines and proxies (once), then
 * checks every asset whose decodability is unknown (new imports, relinked content, undone
 * removals) once per content and reports it to the core, so mezzanine decisions use this
 * machine's capabilities. A failed report is retried when the assets change next.
 */
export function useDecodabilityReporter(projectId: string | null, assets: readonly Asset[]): void {
  const client = useCoreClient();
  const started = useRef(new Set<string>());
  const capabilities = useRef<Promise<void> | null>(null);
  useEffect(() => {
    if (projectId === null) {
      return;
    }
    capabilities.current ??= reportCapabilitiesOf(client).catch((error: unknown) => {
      console.warn('[media-bin] reporting decoder capabilities failed', error);
      capabilities.current = null;
    });
    const reported = capabilities.current;
    for (const asset of assets) {
      const key = `${asset.id}:${asset.fingerprint?.sampleHash ?? ''}`;
      if (!needsDecodabilityCheck(asset) || started.current.has(key)) continue;
      started.current.add(key);
      reported
        .then(() => reportDecodabilityOf(client, projectId, asset))
        .catch((error: unknown) => {
          console.warn('[media-bin] reporting decodability failed', error);
          started.current.delete(key);
        });
    }
  }, [client, projectId, assets]);
}
