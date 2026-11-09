import type { JobKind } from '@se/protocol';
import { useStore } from 'zustand';

import { assetJobStatus } from '../services/job-status';
import { jobsStore } from '../store/jobs-store';

/** Where a job of an asset stands, as its latest job of that kind says. */
export type AssetJobState =
  | { readonly kind: 'queued' }
  | { readonly kind: 'running'; readonly progress: number | null }
  | { readonly kind: 'failed'; readonly message: string };

/** Where the transcription of an asset stands. */
export type TranscriptionJobState = AssetJobState;

/**
 * The state of `assetId`'s latest job of kind `jobKind` (queued, running with its progress, or
 * failed), or `null` when none is waiting, running or failed. Read from the media bin's jobs
 * store (fed while the media bin is mounted, i.e. in the editor).
 */
export function useAssetJob(assetId: string | null, jobKind: JobKind): AssetJobState | null {
  const jobs = useStore(jobsStore, (state) => state.jobs);
  if (assetId === null) return null;
  const status = assetJobStatus(jobs, assetId);
  const running = status.running.find((job) => job.kind === jobKind);
  if (running !== undefined) return { kind: 'running', progress: running.progress };
  if (status.queued.includes(jobKind)) return { kind: 'queued' };
  const failed = status.failed.find((job) => job.kind === jobKind);
  return failed === undefined ? null : { kind: 'failed', message: failed.message };
}

/** The state of `assetId`'s latest transcription job (see {@link useAssetJob}). */
export function useTranscriptionJob(assetId: string): TranscriptionJobState | null {
  return useAssetJob(assetId, 'transcription');
}
