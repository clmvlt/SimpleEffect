import type { Conflict, Flicks, Task } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  composerStore,
  connectComposerPersistence,
  navigateHistory,
  parseComposerPreferences,
  resetComposer,
  setComposerText,
} from '../store/composer-store';
import { task } from '../../../shared/ai-tasks/testing/task-fixture';
import { browseHistory, MAX_HISTORY, NOT_BROWSING, rememberPrompt } from './prompt-history';
import { conflictLines } from './revert-conflicts';
import { shortRange, shortTime } from './short-time';
import { taskNotice } from './task-notice';
import { elapsedMs, orderedTasks, totalUsage } from './task-view';
import { usageText } from './usage-format';

describe('prompt history', () => {
  it('remembers prompts once, newest last, within the limit', () => {
    expect(rememberPrompt(['a', 'b'], ' a ')).toEqual(['b', 'a']);
    expect(rememberPrompt(['a'], '   ')).toEqual(['a']);
    const many = Array.from({ length: MAX_HISTORY }, (_, index) => String(index));
    expect(rememberPrompt(many, 'new')).toHaveLength(MAX_HISTORY);
  });

  it('browses with ↑ from an empty field and ↓ back to the draft', () => {
    const entries = ['first', 'second'];
    const up = browseHistory(entries, NOT_BROWSING, '', 'previous');
    expect(up?.value).toBe('second');
    const upAgain = browseHistory(entries, up?.browsing ?? NOT_BROWSING, 'second', 'previous');
    expect(upAgain?.value).toBe('first');
    expect(
      browseHistory(entries, upAgain?.browsing ?? NOT_BROWSING, 'first', 'previous'),
    ).toBeNull();
    const down = browseHistory(entries, upAgain?.browsing ?? NOT_BROWSING, 'first', 'next');
    expect(down?.value).toBe('second');
    expect(browseHistory(entries, down?.browsing ?? NOT_BROWSING, 'second', 'next')).toEqual({
      value: '',
      browsing: NOT_BROWSING,
    });
    expect(browseHistory(entries, NOT_BROWSING, 'typed', 'previous')).toBeNull();
  });

  it('is saved and restored with the model and effort', () => {
    resetComposer();
    const saved = new Map<string, string>();
    const storage = {
      getItem: (key: string) => saved.get(key) ?? null,
      setItem: (key: string, value: string) => saved.set(key, value),
    };
    const stop = connectComposerPersistence(storage);
    composerStore.setState({ history: ['Coupe les silences'], effort: 'low' });
    stop();
    expect(parseComposerPreferences(saved.get('ai.v1') ?? null)).toEqual({
      model: null,
      effort: 'low',
      history: ['Coupe les silences'],
    });
    expect(parseComposerPreferences('{"effort":"extreme","history":[3,"ok"]}')).toEqual({
      model: null,
      effort: null,
      history: ['ok'],
    });
    setComposerText('');
    expect(navigateHistory('previous')).toBe(true);
    expect(composerStore.getState().text).toBe('Coupe les silences');
    resetComposer();
  });
});

describe('task panel helpers', () => {
  const at = (overrides: Partial<Task>) => task(overrides);

  it('orders running, then queued, then the most recent finished tasks', () => {
    const tasks = [
      at({ id: 'done-old', status: { kind: 'completed' }, updatedAt: 5 }),
      at({ id: 'queued', status: { kind: 'queued' }, createdAt: 2 }),
      at({ id: 'running', status: { kind: 'running' }, createdAt: 3 }),
      at({
        id: 'done-new',
        status: { kind: 'failed', failure: 'crashed', message: '' },
        updatedAt: 9,
      }),
    ];
    expect(orderedTasks(tasks).map((each) => each.id)).toEqual([
      'running',
      'queued',
      'done-new',
      'done-old',
    ]);
  });

  it('measures the latest run and adds up the usage', () => {
    const run = {
      runKind: 'initial',
      prompt: 'p',
      queuedAt: 0,
      startedAt: 1000,
      finishedAt: null,
    } as const;
    const running = at({
      status: { kind: 'running' },
      runs: [{ ...run, status: { kind: 'running' } }],
    });
    expect(elapsedMs(running, 6000)).toBe(5000);
    expect(
      elapsedMs(at({ runs: [{ ...run, startedAt: null, status: { kind: 'queued' } }] }), 9),
    ).toBeNull();
    const total = totalUsage([
      at({ usage: { inputTokens: 10, outputTokens: 2, costUsd: null } }),
      at({ usage: { inputTokens: 5, outputTokens: 1, costUsd: 0.25 } }),
    ]);
    expect(total).toEqual({ inputTokens: 15, outputTokens: 3, costUsd: 0.25 });
    expect(usageText({ inputTokens: 12_400, outputTokens: 300, costUsd: 0.0213 }, 'fr')).toEqual({
      tokens: expect.stringMatching(/^12,7\s?k$/u) as string,
      cost: expect.stringMatching(/^0,021\s?\$/u) as string,
    });
  });

  it('writes short times and ranges', () => {
    expect(shortTime(51)).toBe('00:51');
    expect(shortTime(62.5)).toBe('01:02.5');
    expect(shortTime(3725.25)).toBe('1:02:05.25');
    expect(
      shortRange({ start: (51 * 705_600_000) as Flicks, duration: (3 * 705_600_000) as Flicks }),
    ).toBe('00:51 → 00:54');
  });

  it('announces completed and failed runs, not cancelled ones', () => {
    expect(taskNotice(at({ status: { kind: 'completed' }, prompt: 'Titre\n  ici' }), 3)).toEqual({
      tone: 'success',
      number: 3,
      excerpt: 'Titre ici',
    });
    expect(
      taskNotice(at({ status: { kind: 'failed', failure: 'rateLimited', message: '' } }), 1),
    ).toMatchObject({ tone: 'error', failure: 'rateLimited' });
    expect(taskNotice(at({ status: { kind: 'cancelled' } }), 1)).toBeNull();
  });

  it('lists what a revert keeps', () => {
    const conflicts: Conflict[] = [
      { kind: 'itemChanged', item: 'a', taskRevision: 2, currentRevision: 3 },
      { kind: 'itemChanged', item: 'b', taskRevision: 2, currentRevision: null },
      { kind: 'foreignItems', items: ['c', 'd'] },
      { kind: 'cannotApply', code: 'ITEM_NOT_FOUND', message: 'gone' },
    ];
    expect(conflictLines(conflicts).map((line) => line.kind)).toEqual([
      'changed',
      'changed',
      'foreign',
      'cannotApply',
    ]);
    expect(conflictLines(conflicts)[1]).toMatchObject({ removed: true });
  });
});
