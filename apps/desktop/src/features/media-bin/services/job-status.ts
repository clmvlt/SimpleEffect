import type { CoreEvent, JobKind, JobSnapshot } from '@se/protocol';

/** Jobs of the open project by id, as known from `listJobs` and job events. */
export type JobsById = Readonly<Record<string, JobSnapshot>>;

/** What the media bin shows about the jobs of one asset. */
export interface AssetJobStatus {
  /** Running jobs with their progress in `[0, 1]` (`null` when unknown). */
  readonly running: readonly { kind: JobKind; progress: number | null }[];
  /** Kinds waiting for a slot. */
  readonly queued: readonly JobKind[];
  /** Kinds whose latest job failed, with the reason. */
  readonly failed: readonly { kind: JobKind; message: string }[];
}

/** No job to show. */
export const IDLE_STATUS: AssetJobStatus = { running: [], queued: [], failed: [] };

/** The jobs after a job event; other events leave them unchanged (same object). */
export function applyJobEvent(jobs: JobsById, event: CoreEvent): JobsById {
  switch (event.kind) {
    case 'jobQueued':
    case 'jobCompleted':
    case 'jobFailed':
    case 'jobCancelled':
      return { ...jobs, [event.job.id]: event.job };
    case 'jobProgress': {
      const job = jobs[event.jobId];
      if (job === undefined) {
        return jobs;
      }
      return {
        ...jobs,
        [job.id]: { ...job, state: { kind: 'running', progress: event.progress } },
      };
    }
    default:
      return jobs;
  }
}

/** The jobs listed by `listJobs`, by id. */
export function jobsFromList(list: readonly JobSnapshot[]): JobsById {
  return Object.fromEntries(list.map((job) => [job.id, job]));
}

/**
 * The status of one asset: its running and queued jobs, and the kinds whose latest job failed
 * (a later job of the same kind replaces an old failure). Jobs are compared by id (UUIDv7: newer
 * ids sort after older ones).
 */
export function assetJobStatus(jobs: JobsById, assetId: string): AssetJobStatus {
  const latest = new Map<JobKind, JobSnapshot>();
  for (const job of Object.values(jobs)) {
    if (job.assetId !== assetId) continue;
    const known = latest.get(job.kind);
    if (known === undefined || known.id < job.id) latest.set(job.kind, job);
  }
  if (latest.size === 0) {
    return IDLE_STATUS;
  }
  const running: { kind: JobKind; progress: number | null }[] = [];
  const queued: JobKind[] = [];
  const failed: { kind: JobKind; message: string }[] = [];
  for (const job of latest.values()) {
    if (job.state.kind === 'running')
      running.push({ kind: job.kind, progress: job.state.progress });
    else if (job.state.kind === 'queued') queued.push(job.kind);
    else if (job.state.kind === 'failed')
      failed.push({ kind: job.kind, message: job.state.message });
  }
  return { running, queued, failed };
}

/** The id of the latest completed job of `kind` for an asset (changes when outputs are new). */
export function latestCompletion(jobs: JobsById, assetId: string, kind: JobKind): string | null {
  let latest: string | null = null;
  for (const job of Object.values(jobs)) {
    if (job.assetId === assetId && job.kind === kind && job.state.kind === 'completed') {
      if (latest === null || latest < job.id) latest = job.id;
    }
  }
  return latest;
}
