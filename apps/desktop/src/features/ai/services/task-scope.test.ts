import type { Flicks, ItemEntity, TaskScope, TimeRange } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  boundingRange,
  deriveScope,
  redoTargetOf,
  widenedScope,
  type RedoTarget,
  type ScopeInputs,
} from './task-scope';
import { captionKind } from '../../../shared/project-store/testing/session-fixture';

const SECOND = 705_600_000;
const range = (start: number, end: number): TimeRange => ({
  start: (start * SECOND) as Flicks,
  duration: ((end - start) * SECOND) as Flicks,
});

function item(id: string, trackId: string, start: number, end: number): ItemEntity {
  return {
    id,
    trackId,
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: range(start, end),
    linkGroup: null,
    kind: captionKind(id),
  };
}

const NOTHING: ScopeInputs = {
  sequenceId: 'sequence-1',
  inOut: null,
  selection: [],
  phrase: null,
  redo: null,
  wholeSequence: false,
  chosenTracks: null,
};

const REDO: RedoTarget = {
  itemId: 'item-ai',
  sequenceId: 'sequence-1',
  trackId: 'v2',
  range: range(51, 54),
  taskId: 'task-1',
};

describe('deriveScope', () => {
  it('is the whole sequence on every track without anything else', () => {
    expect(deriveScope(NOTHING)).toEqual({
      scope: { sequenceId: 'sequence-1', range: null, tracks: null },
      rangeSource: 'sequence',
      tracksSource: 'all',
    });
  });

  it('takes the in/out span on every track', () => {
    const inOut = { start: (10 * SECOND) as Flicks, end: (20 * SECOND) as Flicks };
    const derived = deriveScope({ ...NOTHING, inOut });
    expect(derived.scope.range).toEqual(range(10, 20));
    expect([derived.rangeSource, derived.scope.tracks]).toEqual(['inOut', null]);
  });

  it('prefers the selection (bounding range and its tracks) over the in/out span', () => {
    const inOut = { start: (0 * SECOND) as Flicks, end: (5 * SECOND) as Flicks };
    const selection = [item('a', 'v1', 12, 14), item('b', 'v2', 13, 18), item('c', 'v1', 11, 12)];
    const derived = deriveScope({ ...NOTHING, inOut, selection });
    expect(derived.scope).toEqual({
      sequenceId: 'sequence-1',
      range: range(11, 18),
      tracks: ['v1', 'v2'],
    });
    expect([derived.rangeSource, derived.tracksSource]).toEqual(['selection', 'selection']);
  });

  it('prefers a range read in the text, then the redo item', () => {
    const phrase = {
      start: (51 * SECOND) as Flicks,
      end: (54 * SECOND) as Flicks,
      text: 'de 51s à 54s',
      index: 0,
    };
    const selection = [item('a', 'v1', 1, 2)];
    const fromText = deriveScope({ ...NOTHING, selection, redo: REDO, phrase });
    expect([fromText.rangeSource, fromText.scope.range, fromText.scope.tracks]).toEqual([
      'text',
      range(51, 54),
      null,
    ]);
    const fromItem = deriveScope({ ...NOTHING, selection, redo: REDO });
    expect([fromItem.rangeSource, fromItem.scope.tracks]).toEqual(['item', ['v2']]);
  });

  it('lets the user ask for the whole sequence and choose tracks', () => {
    const selection = [item('a', 'v1', 1, 2)];
    expect(deriveScope({ ...NOTHING, selection, wholeSequence: true }).scope).toEqual({
      sequenceId: 'sequence-1',
      range: null,
      tracks: null,
    });
    const chosen = deriveScope({ ...NOTHING, selection, chosenTracks: ['v3'] });
    expect([chosen.scope.tracks, chosen.tracksSource]).toEqual([['v3'], 'chosen']);
    const all = deriveScope({ ...NOTHING, selection, chosenTracks: 'all' });
    expect([all.scope.tracks, all.tracksSource]).toEqual([null, 'chosen']);
  });
});

describe('boundingRange', () => {
  it('spans every item, or nothing', () => {
    expect(boundingRange([])).toBeNull();
    expect(boundingRange([item('a', 'v1', 3, 4), item('b', 'v1', 1, 2)])).toEqual(range(1, 4));
  });
});

describe('widenedScope', () => {
  const scope: TaskScope = { sequenceId: 'sequence-1', range: range(50, 55), tracks: ['v2'] };

  it('keeps a scope that already holds the item', () => {
    expect(widenedScope(scope, REDO)).toBeNull();
    expect(widenedScope({ ...scope, range: null, tracks: null }, REDO)).toBeNull();
  });

  it('widens the range and the tracks to reach a moved item', () => {
    const moved: RedoTarget = { ...REDO, trackId: 'v3', range: range(54, 58) };
    expect(widenedScope(scope, moved)).toEqual({
      sequenceId: 'sequence-1',
      range: range(50, 58),
      tracks: ['v2', 'v3'],
    });
  });
});

describe('redoTargetOf', () => {
  it('names the task that made the item', () => {
    const made = { ...item('x', 'v2', 1, 2), origin: { kind: 'aiTask', taskId: 't' } as const };
    expect(redoTargetOf(made, 'sequence-1')).toEqual({
      itemId: 'x',
      sequenceId: 'sequence-1',
      trackId: 'v2',
      range: range(1, 2),
      taskId: 't',
    });
    expect(redoTargetOf(item('y', 'v1', 1, 2), 'sequence-1').taskId).toBeNull();
  });
});
