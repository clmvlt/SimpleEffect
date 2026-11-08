import type { Asset, AssetId, ProjectId, Transcript } from '@se/protocol';
import { useEffect, useEffectEvent } from 'react';
import { useStore } from 'zustand';

import { useCoreClient } from '../../../shared/core';
import { getTranscript } from '../api';
import { setTranscriptEntry, transcriptKey, transcriptsStore } from '../store/transcripts-store';

/** Transcripts of `assets` and whether some are still being fetched. */
export interface AssetTranscripts {
  readonly transcripts: ReadonlyMap<AssetId, Transcript>;
  readonly loading: boolean;
}

/** The cache keys of the assets that have a transcript summary. */
function keysOf(assets: readonly Asset[]): { assetId: AssetId; key: string }[] {
  return assets.flatMap((asset) => {
    const summary = asset.analysis.transcript;
    return summary === null
      ? []
      : [{ assetId: asset.id, key: transcriptKey(asset.id, summary.transcribedAt) }];
  });
}

/**
 * The transcripts of `assets` (those whose summary says they are transcribed), fetched once per
 * version with `getTranscript` and kept in the feature's store.
 */
export function useAssetTranscripts(
  projectId: ProjectId | null,
  assets: readonly Asset[],
): AssetTranscripts {
  const client = useCoreClient();
  const entries = useStore(transcriptsStore, (state) => state.entries);
  const wanted = keysOf(assets);
  const missing = wanted.filter(({ key }) => entries[key] === undefined);
  const missingSignature = missing.map(({ key }) => key).join(',');
  // Reads the latest assets without making them an effect dependency (a new array each render).
  const fetchMissing = useEffectEvent(() => {
    if (projectId === null) return;
    for (const { assetId, key } of keysOf(assets)) {
      if (transcriptsStore.getState().entries[key] !== undefined) continue;
      setTranscriptEntry(key, { status: 'loading' });
      getTranscript(client, projectId, assetId).then(
        (transcript) => {
          setTranscriptEntry(
            key,
            transcript === null ? { status: 'failed' } : { status: 'ready', transcript },
          );
        },
        (error: unknown) => {
          console.warn('[transcript]', error);
          setTranscriptEntry(key, { status: 'failed' });
        },
      );
    }
  });
  useEffect(() => {
    if (missingSignature !== '') fetchMissing();
  }, [missingSignature, projectId]);
  const transcripts = new Map<AssetId, Transcript>();
  let loading = false;
  for (const { assetId, key } of wanted) {
    const entry = entries[key];
    if (entry?.status === 'ready') transcripts.set(assetId, entry.transcript);
    else if (entry === undefined || entry.status === 'loading') loading = true;
  }
  return { transcripts, loading };
}
