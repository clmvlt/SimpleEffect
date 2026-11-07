import type { Flicks, ItemEntity, ItemId, TrackId } from '@se/protocol';

import type { Placement } from './move-placement';
import { itemEnd } from './visible-items';

/** Whether `item` spans `time` with room on both sides (a cut there makes two items). */
export function crosses(item: ItemEntity, time: Flicks): boolean {
  return item.timelineRange.start < time && itemEnd(item) > time;
}

/**
 * The items a cut at `time` splits: the selected items crossing it, or, with nothing selected,
 * every item crossing it on an unlocked track.
 */
export function itemsToSplit(
  items: readonly ItemEntity[],
  selected: ReadonlySet<ItemId>,
  lockedTracks: ReadonlySet<TrackId>,
  time: Flicks,
): ItemId[] {
  const candidates =
    selected.size > 0
      ? items.filter((item) => selected.has(item.id))
      : items.filter((item) => !lockedTracks.has(item.trackId));
  return candidates.filter((item) => crosses(item, time)).map((item) => item.id);
}

/** Placements nudging `items` by `delta` on their tracks; `null` when nothing would move. */
export function nudgePlacements(items: readonly ItemEntity[], delta: Flicks): Placement[] | null {
  if (items.length === 0) return null;
  const earliest = Math.min(...items.map((item) => item.timelineRange.start));
  const shift = Math.max(delta, -earliest);
  if (shift === 0) return null;
  return items.map((item) => ({
    itemId: item.id,
    trackId: item.trackId,
    start: (item.timelineRange.start + shift) as Flicks,
  }));
}

/** Placements of copies of `items` right after their block, on the same tracks. */
export function duplicatePlacements(items: readonly ItemEntity[]): Placement[] {
  if (items.length === 0) return [];
  const start = Math.min(...items.map((item) => item.timelineRange.start));
  const end = Math.max(...items.map(itemEnd));
  return items.map((item) => ({
    itemId: item.id,
    trackId: item.trackId,
    start: (item.timelineRange.start + end - start) as Flicks,
  }));
}

/** Whether `items` can be linked: at least two, not already all in one group. */
export function canLink(items: readonly ItemEntity[]): boolean {
  if (items.length < 2) return false;
  const group = items[0]?.linkGroup ?? null;
  return group === null || items.some((item) => item.linkGroup !== group);
}

/** Whether one of `items` is linked to others. */
export function canUnlink(items: readonly ItemEntity[]): boolean {
  return items.some((item) => item.linkGroup !== null);
}
