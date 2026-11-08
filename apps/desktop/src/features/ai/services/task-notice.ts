import type { AgentFailureKind, Task } from '@se/protocol';

/** Longest prompt excerpt in a notification. */
const MAX_EXCERPT = 120;

/** What to tell when a run of a task ended: a toast, and a system notification when unfocused. */
export type TaskNotice =
  | { readonly tone: 'success'; readonly number: number; readonly excerpt: string }
  | {
      readonly tone: 'error';
      readonly number: number;
      readonly excerpt: string;
      readonly failure: AgentFailureKind;
    };

/** The notice of `task`'s run that just ended; `null` when it was cancelled (the user knows). */
export function taskNotice(task: Task, number: number): TaskNotice | null {
  const prompt = task.runs.at(-1)?.prompt ?? task.prompt;
  const flat = prompt.replace(/\s+/gu, ' ').trim();
  const excerpt = flat.length > MAX_EXCERPT ? flat.slice(0, MAX_EXCERPT - 1) + '…' : flat;
  switch (task.status.kind) {
    case 'completed':
      return { tone: 'success', number, excerpt };
    case 'failed':
      return { tone: 'error', number, excerpt, failure: task.status.failure };
    default:
      return null;
  }
}
