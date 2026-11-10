import type { ConnectionState, CoreEvent, ExportInfo, Flicks } from '@se/protocol';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { exportsStore, resetExports } from '../store/exports-store';
import { connectExportsSync, type ExportsSyncSource } from './exports-sync';

function info(id: string, state: ExportInfo['state']): ExportInfo {
  return {
    id,
    jobId: null,
    projectId: 'project-1',
    sequenceId: 'sequence-1',
    sequenceName: 'Montage',
    presetName: 'YouTube',
    path: `/exports/${id}.mp4`,
    settings: { container: 'wav', video: null, audio: null },
    duration: 1000 as Flicks,
    createdAt: 0,
    hasThumbnail: false,
    state,
  };
}

/** A fake core: answers `listExports` with `listed`, opens on demand, emits events. */
function fakeClient(listed: () => ExportInfo[]) {
  const stateListeners: (() => void)[] = [];
  const handlers = new Map<string, ((event: CoreEvent) => void)[]>();
  const client = {
    state: 'connecting' as ConnectionState,
    onStateChange: (listener: () => void) => {
      stateListeners.push(listener);
      return () => undefined;
    },
    send: vi.fn(() => Promise.resolve({ kind: 'listExports', exports: listed() })),
    on: (kind: string, handler: (event: CoreEvent) => void) => {
      handlers.set(kind, [...(handlers.get(kind) ?? []), handler]);
      return () => {
        handlers.set(
          kind,
          (handlers.get(kind) ?? []).filter((known) => known !== handler),
        );
      };
    },
  };
  return {
    client: client as unknown as ExportsSyncSource,
    send: client.send,
    open: () => {
      client.state = 'open';
      for (const listener of stateListeners) listener();
    },
    emit: (event: CoreEvent) => {
      for (const handler of handlers.get(event.kind) ?? []) handler(event);
    },
  };
}

const ids = () => exportsStore.getState().exports.map((known) => known.id);

afterEach(() => {
  resetExports();
});

describe('connectExportsSync', () => {
  it('lists the exports each time the connection opens', async () => {
    let listed = [info('a', { kind: 'queued' })];
    const core = fakeClient(() => listed);
    const stop = connectExportsSync(core.client, () => undefined);
    expect(core.send).not.toHaveBeenCalled();
    core.open();
    await vi.waitFor(() => {
      expect(ids()).toEqual(['a']);
    });
    listed = [];
    core.open();
    await vi.waitFor(() => {
      expect(ids()).toEqual([]);
    });
    stop();
  });

  it('applies updates with their previous state, then drops the finished ones', async () => {
    const core = fakeClient(() => [info('a', { kind: 'rendering', progress: 0.2, etaSeconds: 4 })]);
    const changes: [string | undefined, string][] = [];
    const stop = connectExportsSync(core.client, (previous, next) => {
      changes.push([previous?.state.kind, next.state.kind]);
    });
    core.open();
    await vi.waitFor(() => {
      expect(ids()).toEqual(['a']);
    });
    const done = info('a', { kind: 'completed', sizeBytes: 10, elapsedSeconds: 2, check: null });
    core.emit({ kind: 'exportUpdated', export: done });
    core.emit({ kind: 'exportUpdated', export: info('b', { kind: 'queued' }) });
    expect(changes).toEqual([
      ['rendering', 'completed'],
      [undefined, 'queued'],
    ]);
    expect(ids()).toEqual(['a', 'b']);
    core.emit({ kind: 'exportsCleared' });
    expect(ids()).toEqual(['b']);
    stop();
    core.emit({ kind: 'exportUpdated', export: info('c', { kind: 'queued' }) });
    expect(ids()).toEqual(['b']);
  });
});
