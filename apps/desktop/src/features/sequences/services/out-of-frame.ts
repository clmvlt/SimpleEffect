import type { Flicks, ItemId } from '@se/protocol';
import { estimatedCaptionBox } from '@se/engine';
import {
  evaluate,
  type AssetTable,
  type EvaluableSequence,
  type FrameGraph,
  type Layer,
  type Size,
} from '@se/engine/evaluate';

import { insideFrame, layerBounds } from './frame-bounds';
import type { TextMeasurer } from './text-extent';

/** A sequence before and after a format change (or a source and its version). */
export interface FrameChange {
  readonly before: EvaluableSequence;
  readonly after: EvaluableSequence;
  readonly assets: AssetTable;
  /** The item of `before` each item of `after` comes from (the same id after a change). */
  readonly origins: ReadonlyMap<ItemId, ItemId>;
  readonly measureText: TextMeasurer;
}

/** The box of a drawn layer: its natural box, the text's measured one, a caption's estimate. */
function boxOf(layer: Layer, measureText: TextMeasurer): Size {
  if (layer.kind === 'text') return measureText(layer);
  if (layer.kind === 'caption') return estimatedCaptionBox(layer);
  return layer.box;
}

/** Whether the layer of `itemId` in `graph` lies inside the frame; `null` when not drawn. */
function drawnInside(graph: FrameGraph, itemId: ItemId, measureText: TextMeasurer): boolean | null {
  const layer = graph.layers.find((candidate) => candidate.itemId === itemId);
  if (layer === undefined) return null;
  return insideFrame(
    layerBounds(layer.transform, boxOf(layer, measureText), graph.frame),
    graph.frame,
  );
}

/** Instants an item is checked at: its first, middle and last instants. */
function sampleTimes(start: Flicks, duration: Flicks): Flicks[] {
  const last = start + Math.max(0, duration - 1);
  return [start, start + Math.floor(duration / 2), last] as Flicks[];
}

/**
 * The items of `after` that leave the frame although their origin in `before` was inside it at
 * the same instant (first, middle or last instant of the item): what a format change pushed
 * out. Items already outside before (entrances, off-screen elements), hidden tracks and clips
 * filling the frame are not reported.
 */
export function itemsLeavingFrame(change: FrameChange): ItemId[] {
  const graphs = new Map<string, FrameGraph>();
  const graphAt = (side: 'before' | 'after', time: Flicks) => {
    const key = `${side}:${String(time)}`;
    let graph = graphs.get(key);
    if (graph === undefined) {
      graph = evaluate(change[side], change.assets, time);
      graphs.set(key, graph);
    }
    return graph;
  };
  const leaving: ItemId[] = [];
  for (const track of change.after.tracks) {
    if (track.kind !== 'video' || track.hidden) continue;
    for (const item of track.items) {
      const origin = change.origins.get(item.id);
      if (origin === undefined) continue;
      const { start, duration } = item.timelineRange;
      const left = sampleTimes(start, duration).some((time) => {
        const was = drawnInside(graphAt('before', time), origin, change.measureText);
        const is = drawnInside(graphAt('after', time), item.id, change.measureText);
        return was === true && is === false;
      });
      if (left) leaving.push(item.id);
    }
  }
  return leaving;
}

/**
 * Pairs the items of a copy with those of its source by position (same track order, same item
 * order: what `duplicateSequence` keeps).
 */
export function positionalOrigins(
  source: EvaluableSequence,
  copy: EvaluableSequence,
): Map<ItemId, ItemId> {
  const origins = new Map<ItemId, ItemId>();
  copy.tracks.forEach((track, trackIndex) => {
    const sourceTrack = source.tracks[trackIndex];
    track.items.forEach((item, itemIndex) => {
      const sourceItem = sourceTrack?.items[itemIndex];
      if (sourceItem !== undefined) origins.set(item.id, sourceItem.id);
    });
  });
  return origins;
}

/** Each item of `sequence` paired with itself (a format change keeps ids). */
export function sameOrigins(sequence: EvaluableSequence): Map<ItemId, ItemId> {
  return new Map(
    sequence.tracks.flatMap((track) => track.items.map((item) => [item.id, item.id] as const)),
  );
}
