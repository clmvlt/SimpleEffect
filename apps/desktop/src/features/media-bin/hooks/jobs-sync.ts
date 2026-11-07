import type { CoreClient } from '@se/protocol';

import { listJobs } from '../api';
import { jobsStore, receiveJobEvent, setJobs } from '../store/jobs-store';

/** What the jobs sync needs from the core client. */
export type JobsSyncClient = Pick<CoreClient, 'send' | 'on'>;

const JOB_EVENTS = [
  'jobQueued',
  'jobProgress',
  'jobCompleted',
  'jobFailed',
  'jobCancelled',
] as const;

/**
 * Follows the jobs of `projectId`: subscribes to job events, then loads the list (which replaces
 * what events said so far: the core answers after sending the events that preceded the answer).
 * Returns the function that stops following.
 */
export function connectJobsSync(client: JobsSyncClient, projectId: string): () => void {
  if (jobsStore.getState().projectId !== projectId) {
    jobsStore.setState({ projectId, jobs: {} });
  }
  const unsubscribers = JOB_EVENTS.map((kind) => client.on(kind, receiveJobEvent));
  listJobs(client, projectId).then(
    (list) => {
      if (jobsStore.getState().projectId === projectId) {
        setJobs(projectId, list);
      }
    },
    (error: unknown) => {
      console.warn('[media-bin] listing jobs failed', error);
    },
  );
  return () => {
    for (const unsubscribe of unsubscribers) unsubscribe();
  };
}
