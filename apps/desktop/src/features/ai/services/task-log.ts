import type { Task, TaskEventRecord, TaskStatus } from '@se/protocol';

/** `tasks` with `task` put in place (replacing the same id) or appended. */
export function upsertTask(tasks: readonly Task[], task: Task): Task[] {
  const index = tasks.findIndex((candidate) => candidate.id === task.id);
  if (index === -1) return [...tasks, task];
  return tasks.map((candidate, at) => (at === index ? task : candidate));
}

/** `events` with `record` added in sequence order (a duplicate sequence is ignored). */
export function appendEvent(
  events: readonly TaskEventRecord[],
  record: TaskEventRecord,
): TaskEventRecord[] {
  if (events.some((event) => event.sequence === record.sequence)) return [...events];
  return [...events, record].sort((left, right) => left.sequence - right.sequence);
}

/** Whether the task waits or runs. */
export function isTaskActive(status: TaskStatus): boolean {
  return status.kind === 'queued' || status.kind === 'running';
}

/** What a user can do with a task now. */
export interface TaskActions {
  readonly cancel: boolean;
  readonly retry: boolean;
  readonly followUp: boolean;
}

/** Cancel while active; retry once failed or cancelled; follow up once its session is known. */
export function taskActions(task: Task): TaskActions {
  const active = isTaskActive(task.status);
  return {
    cancel: active,
    retry: task.status.kind === 'failed' || task.status.kind === 'cancelled',
    followUp: !active && task.agentSessionId !== null,
  };
}

/** One line describing an event of the log (development panel, not translated on purpose). */
export function describeEvent(record: TaskEventRecord): string {
  const { event } = record;
  switch (event.kind) {
    case 'sessionStarted':
      return ['session', event.sessionId, event.model ?? ''].join(' ').trim();
    case 'assistantText':
      return event.text;
    case 'toolCallStarted':
      return ['→', event.tool, event.inputSummary].join(' ');
    case 'toolCallFinished':
      return [event.ok ? '✓' : '✗', event.tool, event.outputSummary].join(' ');
    case 'usage': {
      const parts = [String(event.inputTokens), '→', String(event.outputTokens), 'tokens'];
      if (event.costUsd !== null) parts.push('$' + event.costUsd.toFixed(4));
      if (event.durationMs !== null) parts.push(String(event.durationMs) + ' ms');
      return parts.join(' ');
    }
    case 'completed':
      return '✔ ' + event.summary;
    case 'failed':
      return '✘ ' + event.failure + ': ' + event.message;
  }
}
