import type { AgentFailureKind, RunKind, Task, TaskEventRecord } from '@se/protocol';

import { idsInCall, toolCallLine } from './tool-calls';

/**
 * A task's event log grouped for display: the agent's texts, its tool calls as compact lines
 * (a call and its result are one entry), the prompts of follow-ups and retries, its final answer
 * when it says more than its last text, and failures. Session and usage events are left out
 * (usage is shown from `Task.usage`).
 */
export type TaskLogEntry =
  | {
      readonly kind: 'run';
      readonly key: string;
      readonly run: number;
      readonly runKind: RunKind;
      readonly prompt: string;
    }
  | { readonly kind: 'text'; readonly key: string; readonly run: number; readonly text: string }
  | {
      readonly kind: 'tool';
      readonly key: string;
      readonly run: number;
      readonly tool: string;
      readonly line: string;
      readonly input: string;
      readonly output: string | null;
      readonly state: 'running' | 'ok' | 'error';
      /** Ids its input and output name (items among them can be selected). */
      readonly ids: readonly string[];
    }
  | { readonly kind: 'answer'; readonly key: string; readonly run: number; readonly text: string }
  | {
      readonly kind: 'failed';
      readonly key: string;
      readonly run: number;
      readonly failure: AgentFailureKind;
      readonly message: string;
    };

type ToolEntry = Extract<TaskLogEntry, { kind: 'tool' }>;

/** The entries of one run's events, in order. */
function runEntries(events: readonly TaskEventRecord[]): TaskLogEntry[] {
  const entries: TaskLogEntry[] = [];
  const calls = new Map<string, number>();
  let lastText = '';
  for (const { event, run, sequence } of events) {
    const key = String(sequence);
    switch (event.kind) {
      case 'assistantText':
        lastText = event.text.trim();
        entries.push({ kind: 'text', key, run, text: lastText });
        break;
      case 'toolCallStarted':
        calls.set(event.callId, entries.length);
        entries.push({
          kind: 'tool',
          key,
          run,
          tool: event.tool,
          line: toolCallLine(event.tool, event.inputSummary),
          input: event.inputSummary,
          output: null,
          state: 'running',
          ids: idsInCall(event.inputSummary, null),
        });
        break;
      case 'toolCallFinished': {
        const index = calls.get(event.callId);
        const started = index === undefined ? undefined : (entries[index] as ToolEntry);
        const finished: ToolEntry = {
          kind: 'tool',
          key: started?.key ?? key,
          run,
          tool: started?.tool ?? event.tool,
          line: started?.line ?? toolCallLine(event.tool, ''),
          input: started?.input ?? '',
          output: event.outputSummary,
          state: event.ok ? 'ok' : 'error',
          ids: idsInCall(started?.input ?? '', event.outputSummary),
        };
        if (index === undefined) entries.push(finished);
        else entries[index] = finished;
        break;
      }
      case 'completed':
        if (event.summary.trim() !== '' && event.summary.trim() !== lastText) {
          entries.push({ kind: 'answer', key, run, text: event.summary.trim() });
        }
        break;
      case 'failed':
        entries.push({ kind: 'failed', key, run, failure: event.failure, message: event.message });
        break;
      case 'sessionStarted':
      case 'usage':
        break;
    }
  }
  return entries;
}

/**
 * The log of `task` from its events: each run after the first opens with its prompt (events of a
 * run the task does not list yet still show, without an opening).
 */
export function taskLogEntries(
  task: Task,
  events: readonly TaskEventRecord[],
): readonly TaskLogEntry[] {
  const runs = Math.max(task.runs.length, ...events.map((record) => record.run + 1));
  return Array.from({ length: runs }, (_, index) => {
    const own = runEntries(events.filter((record) => record.run === index));
    const run = task.runs[index];
    if (index === 0 || run === undefined) return own;
    const opening: TaskLogEntry = {
      kind: 'run',
      key: `run-${String(index)}`,
      run: index,
      runKind: run.runKind,
      prompt: run.prompt,
    };
    return [opening, ...own];
  }).flat();
}
