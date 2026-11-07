import type { Flicks, Format, Item, TrackKind } from '@se/protocol';

import type { FrameGraph, Layer, Size } from './frame-graph';
import { designScale } from './geometry';
import { itemLayer, type AssetTable } from './item-layer';
import {
  transitionProgress,
  transitionSpan,
  type EvaluableTransition,
  type TransitionSpan,
} from './transitions';

/** The parts of an item `evaluate` reads (a domain `Item` or a mirror `ItemEntity` both fit). */
export type EvaluableItem = Pick<Item, 'id' | 'timelineRange' | 'kind'>;

/** The parts of a track `evaluate` reads; items sorted by start, never overlapping (domain). */
export interface EvaluableTrack {
  readonly kind: TrackKind;
  readonly hidden: boolean;
  readonly items: readonly EvaluableItem[];
  /** Its cross-fades (none when absent). */
  readonly transitions?: readonly EvaluableTransition[];
}

/**
 * The parts of a sequence `evaluate` reads (a domain `Sequence` fits). Tracks are in stacking
 * order: the first video track is the bottom layer.
 */
export interface EvaluableSequence {
  readonly format: Pick<Format, 'width' | 'height'>;
  readonly tracks: readonly EvaluableTrack[];
}

/** Index of the item of a sorted, non-overlapping list covering `time` (binary search), or −1. */
function indexAt(items: readonly EvaluableItem[], time: Flicks): number {
  let low = 0;
  let high = items.length - 1;
  while (low <= high) {
    const middle = (low + high) >> 1;
    const item = items[middle];
    if (item === undefined) return -1;
    const { start, duration } = item.timelineRange;
    if (time < start) high = middle - 1;
    else if (time >= start + duration) low = middle + 1;
    else return middle;
  }
  return -1;
}

/** A cross dissolve playing at the evaluated time, with its two items. */
interface ActiveDissolve {
  readonly span: TransitionSpan;
  readonly outgoing: EvaluableItem;
  readonly incoming: EvaluableItem;
}

function endOf(item: EvaluableItem): number {
  return item.timelineRange.start + item.timelineRange.duration;
}

/**
 * The dissolve of `track` playing at `time`, given the index of the item covering it: before the
 * cut that item is the outgoing side and the next item the incoming one, after the cut the reverse.
 */
function activeDissolve(track: EvaluableTrack, index: number, time: Flicks): ActiveDissolve | null {
  const transitions = track.transitions;
  const item = track.items[index];
  if (transitions === undefined || transitions.length === 0 || item === undefined) return null;
  for (const transition of transitions) {
    if (transition.kind !== 'crossDissolve') continue;
    if (transition.leftItem === item.id) {
      const next = track.items[index + 1];
      if (next?.id !== transition.rightItem || next.timelineRange.start !== endOf(item)) continue;
      const span = transitionSpan(transition, endOf(item) as Flicks);
      if (time >= span.start) return { span, outgoing: item, incoming: next };
    } else if (transition.rightItem === item.id) {
      const previous = track.items[index - 1];
      if (previous?.id !== transition.leftItem || endOf(previous) !== item.timelineRange.start)
        continue;
      const span = transitionSpan(transition, item.timelineRange.start);
      if (time < span.end) return { span, outgoing: previous, incoming: item };
    }
  }
  return null;
}

/** What every layer of one evaluation shares. */
interface FrameContext {
  readonly time: Flicks;
  readonly frame: Size;
  readonly designScale: number;
  readonly assets: AssetTable;
}

/** The two layers of a dissolve, weighted `1 − p` and `p`, tagged as one group. */
function dissolveLayers(dissolve: ActiveDissolve, context: FrameContext): Layer[] {
  const progress = transitionProgress(dissolve.span, context.time);
  const sides = [
    { item: dissolve.outgoing, side: 'outgoing', opacity: 1 - progress },
    { item: dissolve.incoming, side: 'incoming', opacity: progress },
  ] as const;
  const layers: Layer[] = [];
  for (const { item, side, opacity } of sides) {
    const layer = itemLayer(item.kind, { ...context, item, opacity });
    if (layer !== null) {
      layers.push({ ...layer, dissolve: { transitionId: dissolve.span.id, side } });
    }
  }
  return layers;
}

/**
 * Everything visible in `sequence` at `time` (sequence flicks), bottom layer first. Pure: the same
 * inputs always give the same frame graph, in preview and in export (golden rule 7).
 *
 * Visible means: on a video track that is not hidden, with `time` in the item's half-open range,
 * drawing something (see `itemLayer`). During a cross dissolve, the track gives both items (the
 * outgoing one past its end, the incoming one before its start) as a dissolve group. Audio tracks
 * draw nothing.
 */
export function evaluate(
  sequence: EvaluableSequence,
  assets: AssetTable,
  time: Flicks,
): FrameGraph {
  const frame = { width: sequence.format.width, height: sequence.format.height };
  const scale = designScale(frame);
  const context: FrameContext = { time, frame, designScale: scale, assets };
  const layers: Layer[] = [];
  for (const track of sequence.tracks) {
    if (track.kind !== 'video' || track.hidden) continue;
    const index = indexAt(track.items, time);
    const item = track.items[index];
    if (item === undefined) continue;
    const dissolve = activeDissolve(track, index, time);
    if (dissolve !== null) {
      layers.push(...dissolveLayers(dissolve, context));
      continue;
    }
    const layer = itemLayer(item.kind, { ...context, item });
    if (layer !== null) layers.push(layer);
  }
  return { time, frame, designScale: scale, layers };
}
