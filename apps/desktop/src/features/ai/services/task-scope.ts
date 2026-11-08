import type {
  Flicks,
  ItemEntity,
  ItemId,
  SequenceId,
  TaskId,
  TaskScope,
  TimeRange,
  TrackId,
} from '@se/protocol';

import { rangeEnd } from '../../../shared/ai-tasks';
import type { TimeSpan } from '../../timeline';
import type { TimePhrase } from './time-phrases';

/**
 * The scope of a prompt from what the editor shows (prompt 20): a time range read in the text,
 * the item « Refaire avec l'IA » was asked for, the selected items or the in/out span, or the
 * whole sequence; and the tracks they bring (or the tracks the user chose).
 */

/** Where the scope's time range comes from. */
export type RangeSource = 'text' | 'item' | 'selection' | 'inOut' | 'sequence';

/** Where the scope's tracks come from. */
export type TracksSource = 'all' | 'chosen' | 'selection' | 'item';

/** The item « Refaire avec l'IA » was asked for. */
export interface RedoTarget {
  readonly itemId: ItemId;
  readonly sequenceId: SequenceId;
  readonly trackId: TrackId;
  readonly range: TimeRange;
  /** The task that created it, whose session can be resumed. */
  readonly taskId: TaskId | null;
}

/** What the scope is derived from. */
export interface ScopeInputs {
  readonly sequenceId: SequenceId;
  /** The in/out span of that sequence, if any. */
  readonly inOut: TimeSpan | null;
  /** The selected items of that sequence. */
  readonly selection: readonly ItemEntity[];
  /** A time range read in the prompt and not dismissed. */
  readonly phrase: TimePhrase | null;
  /** The item to redo, when on that sequence. */
  readonly redo: RedoTarget | null;
  /** The user asked for the whole sequence (removing the automatic range chip). */
  readonly wholeSequence: boolean;
  /** Tracks the user chose (`'all'`: every track); `null` lets the range's source decide. */
  readonly chosenTracks: TrackChoice;
}

/** Tracks the user chose in the composer: some, every one, or none chosen (automatic). */
export type TrackChoice = readonly TrackId[] | 'all' | null;

/** A scope and where its parts come from (the chips say it). */
export interface DerivedScope {
  readonly scope: TaskScope;
  readonly rangeSource: RangeSource;
  readonly tracksSource: TracksSource;
}

/** The smallest range holding every item; `null` without items. */
export function boundingRange(items: readonly ItemEntity[]): TimeRange | null {
  if (items.length === 0) return null;
  const start = Math.min(...items.map((item) => item.timelineRange.start));
  const end = Math.max(...items.map((item) => rangeEnd(item.timelineRange)));
  return { start: start as Flicks, duration: (end - start) as Flicks };
}

/** The distinct tracks of `items`, in first-seen order. */
export function tracksOf(items: readonly ItemEntity[]): TrackId[] {
  return [...new Set(items.map((item) => item.trackId))];
}

function spanRange(span: { readonly start: Flicks; readonly end: Flicks }): TimeRange {
  return { start: span.start, duration: (span.end - span.start) as Flicks };
}

/** The range and its source, by priority: text, redo item, selection, in/out, else none. */
function rangeOf(inputs: ScopeInputs): { range: TimeRange | null; source: RangeSource } {
  if (inputs.wholeSequence) return { range: null, source: 'sequence' };
  if (inputs.phrase !== null) return { range: spanRange(inputs.phrase), source: 'text' };
  if (inputs.redo !== null) return { range: inputs.redo.range, source: 'item' };
  const selected = boundingRange(inputs.selection);
  if (selected !== null) return { range: selected, source: 'selection' };
  if (inputs.inOut !== null) return { range: spanRange(inputs.inOut), source: 'inOut' };
  return { range: null, source: 'sequence' };
}

/** The tracks and their source: chosen ones, else those of the item or selection the range uses. */
function tracksFor(
  inputs: ScopeInputs,
  source: RangeSource,
): { tracks: TrackId[] | null; source: TracksSource } {
  const chosen = inputs.chosenTracks;
  if (chosen === 'all') return { tracks: null, source: 'chosen' };
  if (chosen !== null && chosen.length > 0) return { tracks: [...chosen], source: 'chosen' };
  if (source === 'item' && inputs.redo !== null) {
    return { tracks: [inputs.redo.trackId], source: 'item' };
  }
  if (source === 'selection') return { tracks: tracksOf(inputs.selection), source: 'selection' };
  return { tracks: null, source: 'all' };
}

/** The scope of a prompt given what the editor shows. */
export function deriveScope(inputs: ScopeInputs): DerivedScope {
  const range = rangeOf(inputs);
  const tracks = tracksFor(inputs, range.source);
  return {
    scope: { sequenceId: inputs.sequenceId, range: range.range, tracks: tracks.tracks },
    rangeSource: range.source,
    tracksSource: tracks.source,
  };
}

/**
 * The scope a follow-up of a task needs to reach `target` (« Refaire avec l'IA »): the task's own
 * when it already holds the item, else widened to it (bounding range, union of tracks); `null`
 * when no change is needed.
 */
export function widenedScope(scope: TaskScope, target: RedoTarget): TaskScope | null {
  const range = scope.range;
  const coversTime =
    range === null ||
    (target.range.start >= range.start && rangeEnd(target.range) <= rangeEnd(range));
  const coversTrack = scope.tracks === null || scope.tracks.includes(target.trackId);
  if (coversTime && coversTrack) return null;
  const start = range === null ? null : Math.min(range.start, target.range.start);
  const end = range === null ? null : Math.max(rangeEnd(range), rangeEnd(target.range));
  return {
    sequenceId: scope.sequenceId,
    range:
      start === null || end === null
        ? null
        : { start: start as Flicks, duration: (end - start) as Flicks },
    tracks: scope.tracks === null || coversTrack ? scope.tracks : [...scope.tracks, target.trackId],
  };
}

/** The redo target of an item of a sequence: its range and track, and the task that made it. */
export function redoTargetOf(item: ItemEntity, sequenceId: SequenceId): RedoTarget {
  return {
    itemId: item.id,
    sequenceId,
    trackId: item.trackId,
    range: item.timelineRange,
    taskId: item.origin.kind === 'aiTask' ? item.origin.taskId : null,
  };
}
