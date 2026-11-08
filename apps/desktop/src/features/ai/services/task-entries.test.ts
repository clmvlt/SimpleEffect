import type { TaskEventRecord } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { task } from '../../../shared/ai-tasks/testing/task-fixture';
import { taskLogEntries } from './task-entries';
import { idsInCall, inputTime, toolCallLine } from './tool-calls';

const ID = '0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f80';

describe('toolCallLine', () => {
  it.each([
    ['add_text', '{"text":"Abonne-toi","start":51,"end":54}', 'add_text “Abonne-toi” 00:51→00:54'],
    [
      'add_text',
      '{"animation_in":{"preset":"pop"},"end":54,"start":51,"text":"Abonne-toi"}',
      'add_text “Abonne-toi” pop 00:51→00:54',
    ],
    [
      'add_shape',
      '{"shape":"rectangle","start":"00:00:10.000","duration":2}',
      'add_shape rectangle 00:10→',
    ],
    ['trim_item', '{"item_id":"x","start":51.5}', 'trim_item 00:51.5→'],
    ['split_at', '{"item_id":"x","time":"1:02"}', 'split_at @01:02'],
    ['delete_items', '{"item_ids":["a","b","c"]}', 'delete_items ×3'],
    ['get_frames', '{"times":[1,2]}', 'get_frames ×2'],
    ['get_timeline', '{"range":{"start":0,"end":3725}}', 'get_timeline 00:00→1:02:05'],
    ['get_project_overview', '{}', 'get_project_overview'],
    ['add_text', '{"text":"cut', 'add_text {"text":"cut'],
  ])('%s %s → %s', (tool, input, line) => {
    expect(toolCallLine(tool, input)).toBe(line);
  });

  it('reads seconds and timecodes as short times', () => {
    expect(inputTime(51)).toBe('00:51');
    expect(inputTime('51s')).toBe('00:51');
    expect(inputTime('00:00:51.250')).toBe('00:51.25');
    expect(inputTime('soon')).toBeNull();
  });

  it('finds the ids a call names in its input and output', () => {
    const upper = ID.toUpperCase();
    expect(idsInCall(`{"item_id":"${ID}"}`, `{"created":["${upper}"],"items":[]}`)).toEqual([ID]);
  });
});

describe('taskLogEntries', () => {
  const record = (sequence: number, run: number, event: TaskEventRecord['event']) => ({
    sequence,
    run,
    at: sequence,
    event,
  });

  it('pairs tool calls with their results and opens follow-ups with their prompt', () => {
    const followed = task({
      runs: [
        {
          runKind: 'initial',
          prompt: 'Titre',
          status: { kind: 'completed' },
          queuedAt: 0,
          startedAt: 1,
          finishedAt: 2,
        },
        {
          runKind: 'followUp',
          prompt: 'Plus gros',
          status: { kind: 'failed', failure: 'crashed', message: 'boom' },
          queuedAt: 3,
          startedAt: 4,
          finishedAt: 5,
        },
      ],
    });
    const events: TaskEventRecord[] = [
      record(0, 0, { kind: 'sessionStarted', sessionId: 's', model: null }),
      record(1, 0, {
        kind: 'toolCallStarted',
        callId: 'c1',
        tool: 'add_text',
        inputSummary: '{"text":"Hi","start":1,"end":2}',
      }),
      record(2, 0, {
        kind: 'toolCallFinished',
        callId: 'c1',
        tool: 'add_text',
        ok: false,
        outputSummary: 'OUT_OF_SCOPE',
      }),
      record(3, 0, { kind: 'assistantText', text: 'Fait.' }),
      record(4, 0, {
        kind: 'usage',
        inputTokens: 1,
        outputTokens: 1,
        costUsd: null,
        durationMs: null,
      }),
      record(5, 0, { kind: 'completed', summary: 'Fait.' }),
      record(6, 1, { kind: 'failed', failure: 'crashed', message: 'boom' }),
    ];
    const entries = taskLogEntries(followed, events);
    expect(entries.map((entry) => entry.kind)).toEqual(['tool', 'text', 'run', 'failed']);
    expect(entries[0]).toMatchObject({
      state: 'error',
      line: 'add_text “Hi” 00:01→00:02',
      output: 'OUT_OF_SCOPE',
    });
    expect(entries[2]).toMatchObject({ runKind: 'followUp', prompt: 'Plus gros' });
  });

  it('shows a final answer that says more than the last text', () => {
    const events: TaskEventRecord[] = [
      record(0, 0, { kind: 'assistantText', text: 'Je regarde.' }),
      record(1, 0, { kind: 'completed', summary: 'Titre ajouté à 51 s.' }),
    ];
    expect(taskLogEntries(task(), events).map((entry) => entry.kind)).toEqual(['text', 'answer']);
  });
});
