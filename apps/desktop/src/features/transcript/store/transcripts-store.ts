import type { AssetId, Transcript } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

/**
 * Transcripts of the open project's assets, fetched once per version: the key is the asset and
 * its summary's `transcribedAt` (a new transcription changes it, so the panel fetches again).
 */
export interface TranscriptsState {
  readonly entries: Readonly<Record<string, TranscriptEntry>>;
}

/** One fetched (or fetching) transcript version. */
export type TranscriptEntry =
  | { readonly status: 'loading' }
  | { readonly status: 'ready'; readonly transcript: Transcript }
  | { readonly status: 'failed' };

/** The transcripts store. */
export const transcriptsStore = createStore<TranscriptsState>()(() => ({ entries: {} }));

/** The cache key of an asset's transcript version. */
export function transcriptKey(assetId: AssetId, transcribedAt: number): string {
  return `${assetId}@${String(transcribedAt)}`;
}

/** Records `entry` under `key`. */
export function setTranscriptEntry(key: string, entry: TranscriptEntry): void {
  transcriptsStore.setState((state) => ({ entries: { ...state.entries, [key]: entry } }));
}

/** Forgets every transcript (the project changed). */
export function clearTranscripts(): void {
  transcriptsStore.setState({ entries: {} });
}
