import type { ConnectionState, CoreEvent, Flicks, ProjectPatch, Task } from '@se/protocol';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { aiTasksStore, resetAiTasks } from '../../../shared/ai-tasks';
import { task } from '../../../shared/ai-tasks/testing/task-fixture';
import { connectTaskSync, tasksEditedByUser, type TaskSyncSource } from './task-sync';

const SECOND = 705_600_000;

/** A fake core: answers `listTasks`, opens on demand and emits events to the handlers. */
function fakeClient(tasks: Task[]) {
  const stateListeners: (() => void)[] = [];
  const handlers = new Map<string, ((event: CoreEvent) => void)[]>();
  const client = {
    state: 'connecting' as ConnectionState,
    onStateChange: (listener: () => void) => {
      stateListeners.push(listener);
      return () => undefined;
    },
    send: vi.fn(() => Promise.resolve({ kind: 'listTasks', tasks })),
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
    client: client as unknown as TaskSyncSource,
    open: () => {
      client.state = 'open';
      for (const listener of stateListeners) listener();
    },
    emit: (event: CoreEvent) => {
      for (const handler of handlers.get(event.kind) ?? []) handler(event);
    },
  };
}

function patch(actor: ProjectPatch['actor'], start: number): ProjectPatch {
  return {
    projectId: 'project-1',
    revision: 5,
    actor,
    upserts: {
      project: null,
      sequences: [],
      tracks: [
        {
          id: 'v2',
          sequenceId: 'sequence-1',
          kind: 'video',
          name: 'V2',
          muted: false,
          solo: false,
          locked: false,
          hidden: false,
          volumeDb: 0,
          pan: 0,
          itemIds: ['item-ai'],
          transitions: [],
        },
      ],
      items: [
        {
          id: 'item-ai',
          trackId: 'v2',
          revision: 3,
          origin: { kind: 'user' },
          timelineRange: { start: (start * SECOND) as Flicks, duration: SECOND as Flicks },
          linkGroup: null,
          kind: { kind: 'caption', text: 'Abonne-toi' },
        },
      ],
      markers: [],
      assets: [],
    },
    deletes: { sequences: [], tracks: [], items: [], markers: [], assets: [] },
  };
}

const RANGED = {
  sequenceId: 'sequence-1',
  range: { start: (50 * SECOND) as Flicks, duration: (5 * SECOND) as Flicks },
  tracks: null,
};

afterEach(() => {
  resetAiTasks();
});

describe('connectTaskSync', () => {
  it('lists the tasks once connected, then follows them and announces finished runs', async () => {
    const fake = fakeClient([task()]);
    const onRunFinished = vi.fn();
    const stop = connectTaskSync(fake.client, 'project-1', { onRunFinished });
    expect(aiTasksStore.getState().tasks).toEqual([]);
    fake.open();
    await vi.waitFor(() => {
      expect(aiTasksStore.getState().tasks).toHaveLength(1);
    });
    fake.emit({ kind: 'taskUpdated', task: task({ status: { kind: 'running' } }) });
    expect(onRunFinished).not.toHaveBeenCalled();
    const second = task({ id: 'task-2', createdAt: 9 });
    fake.emit({ kind: 'taskUpdated', task: second });
    expect(aiTasksStore.getState().numbers.get('task-2')).toBe(2);
    fake.emit({ kind: 'taskUpdated', task: task({ status: { kind: 'completed' } }) });
    expect(onRunFinished).toHaveBeenCalledOnce();
    fake.emit({ kind: 'taskUpdated', task: task({ id: 'other', projectId: 'project-2' }) });
    expect(aiTasksStore.getState().tasks).toHaveLength(2);
    stop();
    expect(aiTasksStore.getState().projectId).toBeNull();
  });

  it('makes items pulse after AI patches and notes user edits inside running scopes', async () => {
    const fake = fakeClient([task({ status: { kind: 'running' }, scope: RANGED })]);
    connectTaskSync(fake.client, 'project-1', { onRunFinished: () => undefined });
    fake.open();
    await vi.waitFor(() => {
      expect(aiTasksStore.getState().tasks).toHaveLength(1);
    });
    fake.emit({ kind: 'projectPatched', ...patch({ kind: 'aiTask', taskId: 'task-1' }, 51) });
    expect(aiTasksStore.getState().pulses.has('item-ai')).toBe(true);
    expect(aiTasksStore.getState().userEdited.size).toBe(0);
    fake.emit({ kind: 'projectPatched', ...patch({ kind: 'user' }, 52) });
    expect([...aiTasksStore.getState().userEdited]).toEqual(['task-1']);
    fake.emit({
      kind: 'taskUpdated',
      task: task({ status: { kind: 'completed' }, scope: RANGED }),
    });
    expect(aiTasksStore.getState().userEdited.size).toBe(0);
  });
});

describe('tasksEditedByUser', () => {
  it('ignores edits outside running scopes and edits by AI tasks', () => {
    const running = task({ status: { kind: 'running' }, scope: RANGED });
    const queued = task({ id: 'queued', status: { kind: 'queued' }, scope: RANGED });
    expect(tasksEditedByUser(patch({ kind: 'user' }, 52), [running, queued])).toEqual(['task-1']);
    expect(tasksEditedByUser(patch({ kind: 'user' }, 70), [running])).toEqual([]);
    expect(tasksEditedByUser(patch({ kind: 'system' }, 52), [running])).toEqual([]);
    const otherTracks = { ...running, scope: { ...RANGED, tracks: ['v1'] } };
    expect(tasksEditedByUser(patch({ kind: 'user' }, 52), [otherTracks])).toEqual([]);
  });
});
