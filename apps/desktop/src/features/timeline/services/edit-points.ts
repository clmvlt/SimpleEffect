import type { Flicks, ItemEntity } from '@se/protocol';

import { itemEnd } from './visible-items';

/**
 * The edit points of a sequence: its start and every item start and end, sorted and without
 * duplicates (targets of ↑/↓ and of snapping).
 */
export function editPoints(items: Iterable<ItemEntity>): Flicks[] {
  const points = new Set<number>([0]);
  for (const item of items) {
    points.add(item.timelineRange.start);
    points.add(itemEnd(item));
  }
  return [...points].sort((a, b) => a - b) as Flicks[];
}

/** The last edit point before `time`, or `undefined`. */
export function previousEditPoint(points: readonly Flicks[], time: Flicks): Flicks | undefined {
  return points.findLast((point) => point < time);
}

/** The first edit point after `time`, or `undefined`. */
export function nextEditPoint(points: readonly Flicks[], time: Flicks): Flicks | undefined {
  return points.find((point) => point > time);
}

/** End of the last item: the sequence's duration (zero when empty). */
export function contentEnd(items: Iterable<ItemEntity>): Flicks {
  let end = 0;
  for (const item of items) end = Math.max(end, itemEnd(item));
  return end as Flicks;
}

/**
 * The candidate closest to `time` within `threshold`, or `time` itself (snapping the playhead to
 * edit points, markers and in/out points while it is dragged).
 */
export function snapTime(time: Flicks, candidates: Iterable<Flicks>, threshold: Flicks): Flicks {
  let best = time;
  let bestDistance: number = threshold;
  for (const candidate of candidates) {
    const distance = Math.abs(candidate - time);
    if (distance <= bestDistance) {
      best = candidate;
      bestDistance = distance;
    }
  }
  return best;
}
