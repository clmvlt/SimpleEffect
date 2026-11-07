import type { Command, CoreEvent, JobSnapshot } from '@se/protocol';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { clearJobs, jobsStore } from '../store/jobs-store';
import { testAsset } from '../testing/assets';
import { reportDecodabilityOf } from './decodability-reporter';
import { connectJobsSync, type JobsSyncClient } from './jobs-sync';

const job = (id: string, state: JobSnapshot['state']): JobSnapshot => ({
  id,
  projectId: 'project',
  assetId: 'asset',
  kind: 'thumbnails',
  state,
  attempt: 1,
});

/** A client answering `listJobs` with `list` and recording event subscriptions. */
function fakeClient(list: JobSnapshot[]) {
  const handlers = new Map<string, (event: CoreEvent) => void>();
  const sent: Command[] = [];
  const client = {
    send: vi.fn((command: Command) => {
      sent.push(command);
      return Promise.resolve({ kind: 'listJobs', jobs: list });
    }),
    on: vi.fn((kind: string, handler: (event: CoreEvent) => void) => {
      handlers.set(kind, handler);
      return () => handlers.delete(kind);
    }),
  };
  return { client: client as unknown as JobsSyncClient, handlers, sent };
}

afterEach(() => {
  clearJobs();
});

describe('connectJobsSync', () => {
  it('loads the jobs, then follows events of the project only', async () => {
    const { client, handlers } = fakeClient([job('j1', { kind: 'queued' })]);
    const stop = connectJobsSync(client, 'project');
    await vi.waitFor(() => {
      expect(Object.keys(jobsStore.getState().jobs)).toEqual(['j1']);
    });
    handlers.get('jobCompleted')?.({ kind: 'jobCompleted', job: job('j1', { kind: 'completed' }) });
    expect(jobsStore.getState().jobs.j1?.state).toEqual({ kind: 'completed' });
    handlers.get('jobQueued')?.({
      kind: 'jobQueued',
      job: { ...job('other', { kind: 'queued' }), projectId: 'elsewhere' },
    });
    expect(Object.keys(jobsStore.getState().jobs)).toEqual(['j1']);
    stop();
    expect(handlers.size).toBe(0);
  });
});

describe('reportDecodabilityOf', () => {
  it('reports what the check found for each stream', async () => {
    const send = vi.fn(() => Promise.resolve({ kind: 'reportDecodability' }));
    const asset = testAsset({});
    const check = vi.fn(() =>
      Promise.resolve({ video: 'decodable' as const, audio: 'undecodable' as const }),
    );
    await reportDecodabilityOf({ send } as never, 'project', asset, check);
    expect(check).toHaveBeenCalledWith('video', asset.mediaInfo);
    expect(send).toHaveBeenCalledWith({
      kind: 'reportDecodability',
      projectId: 'project',
      assetId: asset.id,
      video: 'decodable',
      audio: 'undecodable',
    });
  });
});
