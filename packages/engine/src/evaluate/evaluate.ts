import type { Flicks, Format, Item, TrackKind } from '@se/protocol';

import type { FrameGraph, Layer } from './frame-graph';
import { designScale } from './geometry';
import { itemLayer, type AssetTable } from './item-layer';

/** The parts of an item `evaluate` reads (a domain `Item` or a mirror `ItemEntity` both fit). */
export type EvaluableItem = Pick<Item, 'id' | 'timelineRange' | 'kind'>;

/** The parts of a track `evaluate` reads; items sorted by start, never overlapping (domain). */
export interface EvaluableTrack {
  readonly kind: TrackKind;
  readonly hidden: boolean;
  readonly items: readonly EvaluableItem[];
}

/**
 * The parts of a sequence `evaluate` reads (a domain `Sequence` fits). Tracks are in stacking
 * order: the first video track is the bottom layer.
 */
export interface EvaluableSequence {
  readonly format: Pick<Format, 'width' | 'height'>;
  readonly tracks: readonly EvaluableTrack[];
}

/** The item of a sorted, non-overlapping list covering `time`, by binary search. */
function itemAt(items: readonly EvaluableItem[], time: Flicks): EvaluableItem | undefined {
  let low = 0;
  let high = items.length - 1;
  while (low <= high) {
    const middle = (low + high) >> 1;
    const item = items[middle];
    if (item === undefined) return undefined;
    const { start, duration } = item.timelineRange;
    if (time < start) high = middle - 1;
    else if (time >= start + duration) low = middle + 1;
    else return item;
  }
  return undefined;
}

/**
 * Everything visible in `sequence` at `time` (sequence flicks), bottom layer first. Pure: the same
 * inputs always give the same frame graph, in preview and in export (golden rule 7).
 *
 * Visible means: on a video track that is not hidden, with `time` in the item's half-open range,
 * drawing something (see `itemLayer`). Audio tracks draw nothing.
 */
export function evaluate(
  sequence: EvaluableSequence,
  assets: AssetTable,
  time: Flicks,
): FrameGraph {
  const frame = { width: sequence.format.width, height: sequence.format.height };
  const scale = designScale(frame);
  const layers: Layer[] = [];
  for (const track of sequence.tracks) {
    if (track.kind !== 'video' || track.hidden) continue;
    const item = itemAt(track.items, time);
    if (item === undefined) continue;
    const layer = itemLayer(item.kind, { item, time, frame, designScale: scale, assets });
    if (layer !== null) layers.push(layer);
  }
  return { time, frame, designScale: scale, layers };
}
