import { afterEach, describe, expect, it, vi } from 'vitest';

import { task } from './testing/task-fixture';
import { sidePanelStore } from '../side-panel';
import {
  aiTasksStore,
  markUserEdited,
  PULSE_DURATION_MS,
  pulseItems,
  resetAiTasks,
  setProjectTasks,
  showTask,
  upsertMirroredTask,
} from './ai-tasks-store';
import { openComposer, composerRequestStore } from './composer-requests';
import { taskNumbers } from './task-scopes';

afterEach(() => {
  resetAiTasks();
  vi.useRealTimers();
});

describe('shared AI tasks', () => {
  it('numbers tasks from the oldest and keeps the numbers when tasks change', () => {
    const numbers = taskNumbers([task({ id: 'b', createdAt: 5 }), task({ id: 'a', createdAt: 1 })]);
    expect([numbers.get('a'), numbers.get('b')]).toEqual([1, 2]);
    setProjectTasks('project-1', [task({ id: 'a', createdAt: 1 })]);
    upsertMirroredTask(task({ id: 'b', createdAt: 5 }));
    upsertMirroredTask(task({ id: 'a', createdAt: 1, status: { kind: 'running' } }));
    upsertMirroredTask(task({ id: 'c', projectId: 'project-2' }));
    expect(aiTasksStore.getState().tasks.map((each) => each.id)).toEqual(['a', 'b']);
    expect(aiTasksStore.getState().numbers.get('b')).toBe(2);
  });

  it('forgets user edits once the task is over and another project is mirrored', () => {
    setProjectTasks('project-1', [task({ status: { kind: 'running' } })]);
    markUserEdited(['task-1']);
    expect(aiTasksStore.getState().userEdited.has('task-1')).toBe(true);
    upsertMirroredTask(task({ status: { kind: 'completed' } }));
    expect(aiTasksStore.getState().userEdited.size).toBe(0);
    markUserEdited(['task-1']);
    setProjectTasks('project-2', []);
    expect(aiTasksStore.getState().userEdited.size).toBe(0);
  });

  it('lets pulses end after their duration (a newer pulse of the item survives)', () => {
    vi.useFakeTimers();
    pulseItems(['a', 'b']);
    vi.advanceTimersByTime(PULSE_DURATION_MS / 2);
    pulseItems(['b']);
    vi.advanceTimersByTime(PULSE_DURATION_MS / 2);
    expect([...aiTasksStore.getState().pulses.keys()]).toEqual(['b']);
    vi.advanceTimersByTime(PULSE_DURATION_MS);
    expect(aiTasksStore.getState().pulses.size).toBe(0);
  });

  it('shows the AI panel for a task or the composer', () => {
    sidePanelStore.setState({ tab: 'inspector' });
    showTask('task-1');
    const first = aiTasksStore.getState().focus;
    showTask('task-1');
    expect(aiTasksStore.getState().focus?.nonce).not.toBe(first?.nonce);
    expect(sidePanelStore.getState().tab).toBe('ai');
    sidePanelStore.setState({ tab: 'inspector' });
    openComposer('item-1');
    expect(composerRequestStore.getState().request?.itemId).toBe('item-1');
    expect(sidePanelStore.getState().tab).toBe('ai');
  });
});
