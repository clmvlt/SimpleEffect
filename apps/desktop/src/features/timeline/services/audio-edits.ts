import type {
  Flicks,
  FrameRate,
  ItemEntity,
  NewTransition,
  TimeRange,
  TrackEntity,
  Transition,
  TransitionAlignment,
} from '@se/protocol';
import { trackTransitionSpans, type TransitionSpan } from '@se/engine';

import { snapToFrame } from '../../../shared/time';

/** Which fade of a clip: at its start or at its end. */
export type FadeEdge = 'in' | 'out';

/** The media-clip part of an item, or `null` for other kinds. */
export function mediaClipOf(item: ItemEntity) {
  return item.kind.kind === 'mediaClip' ? item.kind : null;
}

/**
 * The length of the `edge` fade of `item` after dragging its handle by `delta` (whole frames,
 * never negative, never overlapping the other fade).
 */
export function fadeLength(
  item: ItemEntity,
  edge: FadeEdge,
  delta: number,
  rate: FrameRate,
): Flicks {
  const clip = mediaClipOf(item);
  if (clip === null) return 0 as Flicks;
  const current = edge === 'in' ? clip.fadeIn : clip.fadeOut;
  const other = edge === 'in' ? clip.fadeOut : clip.fadeIn;
  const wanted = current + (edge === 'in' ? delta : -delta);
  const room = item.timelineRange.duration - other;
  return snapToFrame(Math.min(room, Math.max(0, wanted)) as Flicks, rate, 'nearest');
}

/** A transition of a track as the lanes draw it: its span and the transition. */
export interface TransitionMark {
  readonly transition: Transition;
  readonly span: TransitionSpan;
}

/** The transitions of `track` whose items touch, with their spans. */
export function transitionMarks(
  track: TrackEntity,
  items: readonly ItemEntity[],
): TransitionMark[] {
  const byId = new Map(track.transitions.map((transition) => [transition.id, transition]));
  return trackTransitionSpans(items, track.transitions).flatMap((span) => {
    const transition = byId.get(span.id);
    return transition === undefined ? [] : [{ transition, span }];
  });
}

/** A cut of a track: two items touching, with no transition yet. */
export interface Cut {
  readonly left: ItemEntity;
  readonly right: ItemEntity;
  readonly time: Flicks;
}

/** The cuts of a track without a transition (its items sorted by start). */
export function openCuts(track: TrackEntity, items: readonly ItemEntity[]): Cut[] {
  const joined = new Set(track.transitions.map((transition) => transition.leftItem));
  const cuts: Cut[] = [];
  items.forEach((item, index) => {
    const next = items[index + 1];
    const end = item.timelineRange.start + item.timelineRange.duration;
    if (next !== undefined && next.timelineRange.start === end && !joined.has(item.id)) {
      cuts.push({ left: item, right: next, time: end as Flicks });
    }
  });
  return cuts;
}

/** The cross-fade wanted on `cut`, of `duration`. */
export function transitionOn(
  track: TrackEntity,
  cut: Cut,
  duration: Flicks,
  alignment: TransitionAlignment = 'center',
): NewTransition {
  return {
    trackId: track.id,
    leftItemId: cut.left.id,
    rightItemId: cut.right.id,
    duration,
    alignment,
  };
}

/**
 * The transitions `Ctrl/⌘+T` adds: on each unlocked track (only the tracks of the selected items
 * when some are selected), the cut nearest the playhead within `duration` of it.
 */
export function transitionsAtPlayhead(
  tracks: readonly TrackEntity[],
  itemsByTrack: ReadonlyMap<string, readonly ItemEntity[]>,
  playhead: Flicks,
  duration: Flicks,
  selectedTracks: ReadonlySet<string>,
): NewTransition[] {
  const wanted: NewTransition[] = [];
  for (const track of tracks) {
    if (track.locked || (selectedTracks.size > 0 && !selectedTracks.has(track.id))) continue;
    let nearest: Cut | null = null;
    for (const cut of openCuts(track, itemsByTrack.get(track.id) ?? [])) {
      const distance = Math.abs(cut.time - playhead);
      if (
        distance <= duration &&
        (nearest === null || distance < Math.abs(nearest.time - playhead))
      )
        nearest = cut;
    }
    if (nearest !== null) wanted.push(transitionOn(track, nearest, duration));
  }
  return wanted;
}

/**
 * The duration a transition edge dragged to `time` gives: the distance to the other edge for
 * start/end alignments, twice the distance to the cut when centred; whole frames, at least one.
 */
export function draggedTransitionDuration(
  span: TransitionSpan,
  alignment: TransitionAlignment,
  edge: 'start' | 'end',
  time: number,
  rate: FrameRate,
): Flicks {
  let duration: number;
  if (alignment === 'center') duration = 2 * Math.abs(time - span.cut);
  else if (edge === 'start') duration = span.end - time;
  else duration = time - span.start;
  const frame = snapToFrame(Math.max(0, duration) as Flicks, rate, 'nearest');
  const oneFrame = snapToFrame(1 as Flicks, rate, 'ceil');
  return Math.max(frame, oneFrame) as Flicks;
}

/** A silent stretch drawn on an item: item-relative times `[start, end)` in flicks. */
export interface ItemSilence {
  readonly start: number;
  readonly end: number;
}

/** The silences (source time) of a clip's asset that fall inside the clip, item-relative. */
export function silencesInItem(item: ItemEntity, silences: readonly TimeRange[]): ItemSilence[] {
  const clip = mediaClipOf(item);
  if (clip === null) return [];
  const duration = item.timelineRange.duration;
  const local = (source: number) => (source - clip.sourceStart) / clip.speed;
  return silences
    .map((silence) => ({
      start: Math.max(0, local(silence.start)),
      end: Math.min(duration, local(silence.start + silence.duration)),
    }))
    .filter((silence) => silence.end > silence.start);
}
