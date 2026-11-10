import type { ExportSession, ExportSessionOptions, ExportSessionResult } from '@se/engine/export';
import type { ExportRenderRequest, Flicks } from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import { IDS, session } from '../../../shared/project-store/testing/session-fixture';
import { snapshotScene, startExportRender, type ExportRenderDeps } from './export-render';

const NO_ISSUES: ExportSessionResult = { missingAssets: [], missingFonts: [], failedTemplates: [] };

const request: ExportRenderRequest = {
  exportId: 'export-1',
  projectId: IDS.project,
  sequenceId: IDS.sequence,
  snapshot: session().snapshot,
  settings: { container: 'wav', video: null, audio: null },
  start: 0 as Flicks,
  end: 1000 as Flicks,
  pipeline: { kind: 'muxed' },
};

/** A session the test ends by hand, recording what it was given. */
function fakeSession() {
  let resolve: (result: ExportSessionResult) => void = () => undefined;
  let reject: (error: Error) => void = () => undefined;
  const done = new Promise<ExportSessionResult>((onResolve, onReject) => {
    resolve = onResolve;
    reject = onReject;
  });
  const cancel = vi.fn(() => {
    const stopped = new Error('the export was stopped');
    stopped.name = 'ExportStoppedError';
    reject(stopped);
  });
  const value: ExportSession = { done, cancel };
  return { value, cancel, resolve, reject };
}

function deps(session: ExportSession, overrides: Partial<ExportRenderDeps> = {}) {
  let clock = 0;
  const options: ExportSessionOptions[] = [];
  const reportProgress = vi.fn<ExportRenderDeps['reportProgress']>(() => Promise.resolve());
  const resolved = {
    endpoint: () => ({ url: 'http://127.0.0.1:1', token: 't' }),
    fonts: vi.fn(() => Promise.resolve([])),
    startSession: vi.fn((given: ExportSessionOptions) => {
      options.push(given);
      return Promise.resolve(session);
    }),
    reportProgress,
    finish: vi.fn(() => Promise.resolve()),
    fail: vi.fn(() => Promise.resolve()),
    onIssues: vi.fn(),
    now: () => clock,
    ...overrides,
  };
  return { resolved, reportProgress, options, tick: (ms: number) => (clock += ms) };
}

describe('startExportRender', () => {
  it('draws the snapshot sequence, throttles progress and reports the end', async () => {
    const fake = fakeSession();
    const { resolved, reportProgress, options, tick } = deps(fake.value);
    const render = startExportRender(request, resolved);
    await vi.waitFor(() => {
      expect(options).toHaveLength(1);
    });
    const job = options[0]?.job;
    expect(job).toMatchObject({ exportId: 'export-1', start: 0, end: 1000 });
    expect(job?.frameRate).toEqual({ numerator: 25, denominator: 1 });
    const onProgress = options[0]?.onProgress ?? (() => undefined);
    onProgress(1, 10);
    onProgress(2, 10);
    tick(300);
    onProgress(3, 10);
    onProgress(10, 10);
    expect(reportProgress.mock.calls).toEqual([
      ['export-1', 1, 10],
      ['export-1', 3, 10],
      ['export-1', 10, 10],
    ]);
    const issues = { ...NO_ISSUES, missingFonts: ['Gone'] };
    fake.resolve(issues);
    await render.done;
    expect(resolved.finish).toHaveBeenCalledWith('export-1');
    expect(resolved.onIssues).toHaveBeenCalledWith('export-1', issues);
    expect(resolved.fail).not.toHaveBeenCalled();
  });

  it('reports a failure with its reason', async () => {
    const fake = fakeSession();
    const { resolved } = deps(fake.value);
    const render = startExportRender(request, resolved);
    await vi.waitFor(() => {
      expect(resolved.startSession).toHaveBeenCalled();
    });
    fake.reject(new Error('the encoder refused the frame'));
    await render.done;
    expect(resolved.fail).toHaveBeenCalledWith('export-1', 'the encoder refused the frame');
    expect(resolved.finish).not.toHaveBeenCalled();
  });

  it('stops silently when cancelled, even before the session started', async () => {
    const running = fakeSession();
    const first = deps(running.value);
    const render = startExportRender(request, first.resolved);
    await vi.waitFor(() => {
      expect(first.resolved.startSession).toHaveBeenCalled();
    });
    render.cancel();
    await render.done;
    expect(running.cancel).toHaveBeenCalled();
    expect(first.resolved.fail).not.toHaveBeenCalled();

    const early = fakeSession();
    const second = deps(early.value);
    const cancelledEarly = startExportRender(request, second.resolved);
    cancelledEarly.cancel();
    await cancelledEarly.done;
    expect(early.cancel).toHaveBeenCalled();
    expect(second.resolved.finish).not.toHaveBeenCalled();
    expect(second.resolved.fail).not.toHaveBeenCalled();
  });

  it('fails without a connection or a sequence', async () => {
    const { resolved } = deps(fakeSession().value, { endpoint: () => null });
    await startExportRender(request, resolved).done;
    expect(resolved.fail).toHaveBeenCalledWith('export-1', expect.stringMatching(/connection/));
    expect(() => snapshotScene(request.snapshot, 'other')).toThrow(/sequence/);
  });
});
