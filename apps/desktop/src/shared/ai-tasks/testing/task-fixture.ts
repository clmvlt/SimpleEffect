import type { Task } from '@se/protocol';

/** A queued task of the fixture project `project-1`, with `overrides`. */
export function task(overrides: Partial<Task> = {}): Task {
  return {
    id: 'task-1',
    projectId: 'project-1',
    prompt: 'Cut the silences',
    scope: { sequenceId: 'sequence-1', range: null, tracks: null },
    provider: 'claude-code',
    model: null,
    effort: null,
    status: { kind: 'queued' },
    agentSessionId: null,
    changesetId: null,
    createdAt: 1,
    updatedAt: 1,
    runs: [],
    usage: { inputTokens: 0, outputTokens: 0, costUsd: null },
    ...overrides,
  };
}
