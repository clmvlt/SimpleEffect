import type { CoreEvent } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

import { applyJobEvent, jobsFromList, type JobsById } from '../services/job-status';

/** Background jobs of the open project, as the media bin shows them. */
export interface JobsState {
  /** The project whose jobs are held, or `null`. */
  readonly projectId: string | null;
  readonly jobs: JobsById;
}

/** The jobs store (fed by `listJobs` and job events, see `hooks/jobs-sync.ts`). */
export const jobsStore = createStore<JobsState>()(() => ({ projectId: null, jobs: {} }));

/** Replaces the jobs with those listed for `projectId`. */
export function setJobs(projectId: string, list: Parameters<typeof jobsFromList>[0]): void {
  jobsStore.setState({ projectId, jobs: jobsFromList(list) });
}

/** Applies a job event if it concerns the project held. */
export function receiveJobEvent(event: CoreEvent): void {
  const { projectId, jobs } = jobsStore.getState();
  if ('projectId' in event && event.projectId === projectId) {
    jobsStore.setState({ jobs: applyJobEvent(jobs, event) });
    return;
  }
  if ('job' in event && event.job.projectId === projectId) {
    jobsStore.setState({ jobs: applyJobEvent(jobs, event) });
  }
}

/** Forgets every job (the project closed). */
export function clearJobs(): void {
  jobsStore.setState({ projectId: null, jobs: {} });
}
