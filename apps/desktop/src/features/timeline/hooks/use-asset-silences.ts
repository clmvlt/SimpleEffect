import type { Asset, CoreClient, TimeRange } from '@se/protocol';
import { useEffect, useSyncExternalStore } from 'react';

import { useCoreClient } from '../../../shared/core';
import { useProjectSelector } from '../../../shared/project-store';
import { timelineApi } from '../api';

/**
 * Silences of assets, fetched once per analysis result (`getAudioSilences`) and shared by every
 * clip of the same asset. The key includes the asset's analysis summary, so a new analysis (a
 * relinked file) fetches again.
 */
type Entry = readonly TimeRange[] | 'loading';

const entries = new Map<string, Entry>();
const listeners = new Set<() => void>();
const NONE: readonly TimeRange[] = [];

function subscribe(listener: () => void): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

/** The cache key of an asset's silences, or `null` when it has none to show. */
function keyOf(projectId: string | null, asset: Asset | null): string | null {
  const summary = asset?.analysis.audio;
  if (projectId === null || asset === null || summary === null || summary === undefined) {
    return null;
  }
  if (summary.silenceCount === 0) return null;
  return [projectId, asset.id, summary.silenceCount, summary.silenceDuration].join(':');
}

function load(client: Pick<CoreClient, 'send'>, projectId: string, assetId: string, key: string) {
  if (entries.has(key)) return;
  entries.set(key, 'loading');
  timelineApi(client, projectId)
    .getAudioSilences(assetId)
    .then((silences) => entries.set(key, silences ?? NONE))
    .catch(() => entries.set(key, NONE))
    .finally(() => {
      for (const listener of listeners) listener();
    });
}

/** The silences of `asset` (source time) when `enabled`, empty while unknown or loading. */
export function useAssetSilences(asset: Asset | null, enabled: boolean): readonly TimeRange[] {
  const client = useCoreClient();
  const projectId = useProjectSelector((state) => state.mirror?.projectId ?? null);
  const key = enabled ? keyOf(projectId, asset) : null;
  useEffect(() => {
    if (key !== null && projectId !== null && asset !== null) {
      load(client, projectId, asset.id, key);
    }
  }, [client, key, projectId, asset]);
  const entry = useSyncExternalStore(subscribe, () =>
    key === null ? undefined : entries.get(key),
  );
  return entry === undefined || entry === 'loading' ? NONE : entry;
}
