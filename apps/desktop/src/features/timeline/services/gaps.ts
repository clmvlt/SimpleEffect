import type { Flicks, ItemEntity, ItemId, TrackEntity, TrackId } from '@se/protocol';

import type { PreviewPlacement } from './preview';
import { concernedTrackIds, shiftedPlacements } from './ripple-preview';
import { itemEnd } from './visible-items';

/** Empty time on a track before an item: from the previous item's end (or 0) to its start. */
export interface Gap {
  readonly trackId: TrackId;
  readonly start: Flicks;
  readonly end: Flicks;
}

/**
 * The gap of a track at `time` (`items` sorted by start), or `null` when an item is there or
 * nothing follows (the free time after the last item is not a gap).
 */
export function gapAt(items: readonly ItemEntity[], trackId: TrackId, time: Flicks): Gap | null {
  let start = 0;
  for (const item of items) {
    const itemStart = item.timelineRange.start;
    if (time < itemStart) {
      return itemStart > start ? { trackId, start: start as Flicks, end: itemStart } : null;
    }
    if (time < itemEnd(item)) return null;
    start = itemEnd(item);
  }
  return null;
}

/** Whether `gap` is still a gap of `items` (nothing in it, an item starting at its end). */
export function isGapOf(items: readonly ItemEntity[], gap: Gap): boolean {
  const current = gapAt(items, gap.trackId, gap.start);
  return current !== null && current.start === gap.start && current.end === gap.end;
}

/** Length of `gap`. */
export function gapDuration(gap: Gap): Flicks {
  return (gap.end - gap.start) as Flicks;
}

/** How a gap resize shows: what follows the gap moves on its track and the linked tracks. */
export function gapResizePreview(
  gap: Gap,
  duration: Flicks,
  tracks: readonly TrackEntity[],
  itemsByTrack: ReadonlyMap<TrackId, readonly ItemEntity[]>,
): Map<ItemId, PreviewPlacement> {
  const concerned = concernedTrackIds(tracks, itemsByTrack, new Set([gap.trackId]), gap.end);
  return shiftedPlacements(
    itemsByTrack,
    concerned,
    gap.end,
    (duration - gapDuration(gap)) as Flicks,
  );
}
