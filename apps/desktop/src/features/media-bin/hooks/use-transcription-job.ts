import { useStore } from 'zustand';

import { assetJobStatus } from '../services/job-status';
import { jobsStore } from '../store/jobs-store';

/** Where the transcription of an asset stands, as its latest job says. */
export type TranscriptionJobState =
  | { readonly kind: 'queued' }
  | { readonly kind: 'running'; readonly progress: number | null }
  | { readonly kind: 'failed'; readonly message: string };

/**
 * The state of `assetId`'s latest transcription job (queued, running with its progress, or
 * failed), or `null` when none is waiting, running or failed. Read from the media bin's jobs
 * store (fed while the media bin is mounted, i.e. in the editor).
 */
export function useTranscriptionJob(assetId: string): TranscriptionJobState | null {
  const jobs = useStore(jobsStore, (state) => state.jobs);
  const status = assetJobStatus(jobs, assetId);
  const running = status.running.find((job) => job.kind === 'transcription');
  if (running !== undefined) return { kind: 'running', progress: running.progress };
  if (status.queued.includes('transcription')) return { kind: 'queued' };
  const failed = status.failed.find((job) => job.kind === 'transcription');
  return failed === undefined ? null : { kind: 'failed', message: failed.message };
}
