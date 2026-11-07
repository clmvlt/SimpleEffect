import type { Flicks, ItemEntity } from '@se/protocol';

/** End of an item on the timeline (exclusive). */
export function itemEnd(item: ItemEntity): Flicks {
  return (item.timelineRange.start + item.timelineRange.duration) as Flicks;
}

/** Index of the first item ending after `time`; `items` sorted by start, never overlapping. */
function firstEndingAfter(items: readonly ItemEntity[], time: Flicks): number {
  let low = 0;
  let high = items.length;
  while (low < high) {
    const middle = (low + high) >>> 1;
    const item = items[middle];
    if (item === undefined) break;
    if (itemEnd(item) <= time) low = middle + 1;
    else high = middle;
  }
  return low;
}

/**
 * The items of a track that intersect `[start, end)` (virtualisation: only these are rendered).
 * Track items are sorted by start and never overlap, so their ends are sorted too: a binary
 * search finds the first one, then the scan stops at the first item starting after the range.
 */
export function visibleItems(
  items: readonly ItemEntity[],
  start: Flicks,
  end: Flicks,
): ItemEntity[] {
  const visible: ItemEntity[] = [];
  for (let index = firstEndingAfter(items, start); index < items.length; index += 1) {
    const item = items[index];
    if (item === undefined || item.timelineRange.start >= end) break;
    visible.push(item);
  }
  return visible;
}
