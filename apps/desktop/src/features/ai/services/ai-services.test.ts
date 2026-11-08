import type { AiStatus, ConnectionState, CoreEvent, TaskEventRecord } from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import { aiAvailability, aiIndicator, canSubmitTasks } from './ai-indicator';
import { aiTextValues } from './ai-texts';
import { appendEvent, revertOutcome, taskActions } from './task-log';
import { task } from '../../../shared/ai-tasks/testing/task-fixture';
import { watchAiStatus, type AiStatusSource } from './watch-ai-status';

const READY: AiStatus = {
  provider: {
    id: 'claude-code',
    name: 'Claude Code',
    command: 'claude',
    loginCommand: '/login',
    helpUrl: 'https://code.claude.com/docs/en/setup',
  },
  installation: { kind: 'installed', path: '/bin/claude', version: '2.1.317' },
  auth: { kind: 'loggedIn', accountKind: 'max' },
  capabilities: { resumeSessions: true, imageInput: true, models: [], effortLevels: [] },
};

describe('aiIndicator', () => {
  it('is green when logged in, orange when logged out, grey when not installed', () => {
    expect(aiIndicator(READY)).toMatchObject({ availability: 'ready', tone: 'success' });
    const loggedOut: AiStatus = { ...READY, auth: { kind: 'loggedOut' } };
    expect(aiIndicator(loggedOut)).toMatchObject({ availability: 'loggedOut', tone: 'warning' });
    const missing: AiStatus = {
      ...READY,
      installation: { kind: 'notInstalled', message: 'not found' },
      auth: { kind: 'unknown', reason: 'not installed' },
    };
    expect(aiIndicator(missing)).toMatchObject({ availability: 'notInstalled', tone: 'neutral' });
    expect(aiAvailability(null)).toBe('checking');
  });

  it('lets tasks through unless the provider is missing or logged out', () => {
    expect(canSubmitTasks(READY)).toBe(true);
    expect(canSubmitTasks({ ...READY, auth: { kind: 'unknown', reason: '?' } })).toBe(true);
    expect(canSubmitTasks({ ...READY, auth: { kind: 'loggedOut' } })).toBe(false);
    expect(canSubmitTasks(null)).toBe(false);
  });

  it('names the provider, its commands and its version for the texts', () => {
    expect(aiTextValues(READY, 'IA')).toEqual({
      name: 'Claude Code',
      command: 'claude',
      login: '/login',
      version: '2.1.317',
      account: 'max',
      reason: '',
    });
    expect(aiTextValues(null, 'IA').name).toBe('IA');
    const missing: AiStatus = {
      ...READY,
      installation: { kind: 'notInstalled', message: 'not found' },
      auth: { kind: 'loggedOut' },
    };
    expect(aiTextValues(missing, 'IA').reason).toBe('not found');
  });
});

describe('task log', () => {
  it('keeps events in sequence order once', () => {
    const event = (sequence: number): TaskEventRecord => ({
      sequence,
      run: 0,
      at: sequence,
      event: { kind: 'assistantText', text: String(sequence) },
    });
    const events = [event(2), event(0), event(2), event(1)].reduce<TaskEventRecord[]>(
      appendEvent,
      [],
    );
    expect(events.map((record) => record.sequence)).toEqual([0, 1, 2]);
  });

  it('offers cancel while active, retry after a failure, follow-up once a session exists', () => {
    expect(taskActions(task())).toEqual({
      cancel: true,
      retry: false,
      followUp: false,
      revert: false,
      changes: false,
    });
    const failed = task({
      status: { kind: 'failed', failure: 'rateLimited', message: 'later' },
      agentSessionId: 'session',
    });
    expect(taskActions(failed)).toEqual({
      cancel: false,
      retry: true,
      followUp: true,
      revert: false,
      changes: false,
    });
    expect(taskActions(failed, false).followUp).toBe(false);
    expect(taskActions(task({ status: { kind: 'completed' } })).followUp).toBe(false);
    const edited = task({ status: { kind: 'completed' }, changesetId: 'changeset-1' });
    expect(taskActions(edited)).toMatchObject({ revert: true, changes: true });
    expect(taskActions({ ...edited, status: { kind: 'running' } }).revert).toBe(false);
  });

  it('sums up what a revert undid and kept', () => {
    const conflict = {
      kind: 'itemChanged',
      item: 'item-1',
      taskRevision: 2,
      currentRevision: 3,
    } as const;
    const report = { taskId: 'task-1', reverted: true, revertedItems: ['item-2'], conflicts: [] };
    expect(revertOutcome(report)).toEqual({ kind: 'reverted' });
    expect(revertOutcome({ ...report, conflicts: [conflict] })).toEqual({
      kind: 'partial',
      kept: 1,
    });
    expect(revertOutcome({ ...report, reverted: false, conflicts: [conflict] })).toEqual({
      kind: 'nothing',
      kept: 1,
    });
  });
});

describe('watchAiStatus', () => {
  function fakeClient(state: ConnectionState) {
    const stateListeners: (() => void)[] = [];
    let eventHandler: ((event: CoreEvent) => void) | null = null;
    const client = {
      state,
      onStateChange: (listener: () => void) => {
        stateListeners.push(listener);
        return () => undefined;
      },
      send: vi.fn(() => Promise.resolve({ kind: 'getAiStatus', status: READY })),
      on: (_kind: string, handler: (event: CoreEvent) => void) => {
        eventHandler = handler;
        return () => undefined;
      },
    };
    return {
      client: client as unknown as AiStatusSource,
      raw: client,
      open: () => {
        client.state = 'open';
        for (const listener of stateListeners) listener();
      },
      emit: (event: CoreEvent) => {
        eventHandler?.(event);
      },
    };
  }

  it('asks the status once connected and follows its changes', async () => {
    const fake = fakeClient('connecting');
    const statuses: (AiStatus | null)[] = [];
    const stop = watchAiStatus(fake.client, (status) => statuses.push(status));
    expect(fake.raw.send).not.toHaveBeenCalled();
    fake.open();
    await vi.waitFor(() => {
      expect(statuses).toEqual([READY]);
    });
    fake.emit({ kind: 'aiStatusChanged', ...READY, auth: { kind: 'loggedOut' } });
    expect(statuses.at(-1)?.auth).toEqual({ kind: 'loggedOut' });
    stop();
    fake.emit({ kind: 'aiStatusChanged', ...READY });
    expect(statuses).toHaveLength(2);
  });
});
