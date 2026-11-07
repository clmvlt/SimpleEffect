import type { Flicks, ItemEntity, ItemId, MarkerEntity } from '@se/protocol';

import { itemEnd } from './visible-items';

/** Pixels within which a dragged edge snaps to a point (the playhead uses the same distance). */
export const SNAP_PIXELS = 8;

/** What dragged edges snap to. */
export interface SnapSources {
  /** Items of the sequence: their edges are snap points. */
  readonly items: Iterable<ItemEntity>;
  /** Items being dragged: their own edges are not snap points. */
  readonly exclude: ReadonlySet<ItemId>;
  readonly playhead: Flicks;
  readonly markers: readonly MarkerEntity[];
  /** The in and out points that are set. */
  readonly inOut: readonly Flicks[];
}

/**
 * The snap points of a sequence, sorted and without duplicates: its start, the edges of items
 * not being dragged, the playhead, markers and in/out points.
 */
export function snapPoints(sources: SnapSources): Flicks[] {
  const points = new Set<number>([0, sources.playhead, ...sources.inOut]);
  for (const marker of sources.markers) points.add(marker.time);
  for (const item of sources.items) {
    if (sources.exclude.has(item.id)) continue;
    points.add(item.timelineRange.start);
    points.add(itemEnd(item));
  }
  return [...points].sort((a, b) => a - b) as Flicks[];
}

/** A snap: the shift to apply and the point an edge lands on. */
export interface Snap {
  readonly delta: Flicks;
  readonly point: Flicks;
}

/** The point of sorted `points` nearest to `time`. */
function nearest(points: readonly Flicks[], time: Flicks): Flicks | undefined {
  let low = 0;
  let high = points.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    if ((points[middle] ?? 0) < time) low = middle + 1;
    else high = middle;
  }
  const after = points[low];
  const before = points[low - 1];
  if (after === undefined) return before;
  if (before === undefined) return after;
  return time - before <= after - time ? before : after;
}

/**
 * The smallest shift bringing one of `edges` onto one of the sorted `points`, if one is within
 * `threshold`; `null` otherwise (the edges stay where the pointer puts them).
 */
export function snapShift(
  edges: readonly Flicks[],
  points: readonly Flicks[],
  threshold: Flicks,
): Snap | null {
  let best: Snap | null = null;
  for (const edge of edges) {
    const point = nearest(points, edge);
    if (point === undefined) continue;
    const delta = point - edge;
    if (Math.abs(delta) <= threshold && (best === null || Math.abs(delta) < Math.abs(best.delta))) {
      best = { delta: delta as Flicks, point };
    }
  }
  return best;
}
