import type { Flicks, ItemEntity, ItemId, TrackEntity } from '@se/protocol';

import { rangeEnd } from '../../../shared/ai-tasks';

/** End of the last item on `tracks` (zero when they are empty). */
export function sequenceContentEnd(
  tracks: readonly TrackEntity[],
  items: Readonly<Record<ItemId, ItemEntity>>,
): Flicks {
  let end = 0;
  for (const track of tracks) {
    for (const id of track.itemIds) {
      const item = items[id];
      if (item !== undefined) end = Math.max(end, rangeEnd(item.timelineRange));
    }
  }
  return end as Flicks;
}
