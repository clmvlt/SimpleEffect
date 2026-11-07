import type { Flicks, ItemEntity, ItemId, TrackEntity, TrackId } from '@se/protocol';

import { trackNumbers, tracksByKind } from './track-numbers';

/** Where an item goes (the `moveItems` / `duplicateItems` placement). */
export interface Placement {
  readonly itemId: ItemId;
  readonly trackId: TrackId;
  readonly start: Flicks;
}

/**
 * How many tracks a drag moves items by: the pointer's track number minus the grabbed item's
 * when both are of the same kind (picture items move among video tracks, sound among audio
 * tracks, by the same count); `previous` while the pointer is over the other kind or no track.
 */
export function dragTrackOffset(
  tracks: readonly TrackEntity[],
  anchorTrackId: TrackId,
  pointerTrackId: TrackId | null,
  previous: number,
): number {
  const numbers = trackNumbers(tracks);
  const anchor = tracks.find((track) => track.id === anchorTrackId);
  const pointer = tracks.find((track) => track.id === pointerTrackId);
  if (anchor === undefined || pointer?.kind !== anchor.kind) return previous;
  return (numbers.get(pointer.id) ?? 0) - (numbers.get(anchor.id) ?? 0);
}

/** `offset` reduced so every item stays on an existing track of its kind. */
export function clampTrackOffset(
  tracks: readonly TrackEntity[],
  items: readonly ItemEntity[],
  offset: number,
): number {
  const numbers = trackNumbers(tracks);
  const byKind = tracksByKind(tracks);
  const kindOf = new Map(tracks.map((track) => [track.id, track.kind]));
  let low = -Infinity;
  let high = Infinity;
  for (const item of items) {
    const kind = kindOf.get(item.trackId);
    const number = numbers.get(item.trackId);
    if (kind === undefined || number === undefined) continue;
    low = Math.max(low, -number);
    high = Math.min(high, byKind[kind].length - 1 - number);
  }
  if (low > high) return 0;
  return Math.min(high, Math.max(low, offset));
}

/** `delta` raised so no item starts before zero. */
export function clampTimeDelta(items: readonly ItemEntity[], delta: Flicks): Flicks {
  const earliest = Math.min(...items.map((item) => item.timelineRange.start));
  return Math.max(delta, -earliest) as Flicks;
}

/** The placements of `items` moved by `delta` in time and `offset` track numbers. */
export function movedPlacements(
  items: readonly ItemEntity[],
  tracks: readonly TrackEntity[],
  delta: Flicks,
  offset: number,
): Placement[] {
  const numbers = trackNumbers(tracks);
  const byKind = tracksByKind(tracks);
  const kindOf = new Map(tracks.map((track) => [track.id, track.kind]));
  return items.map((item) => {
    const kind = kindOf.get(item.trackId) ?? 'video';
    const number = (numbers.get(item.trackId) ?? 0) + offset;
    return {
      itemId: item.id,
      trackId: byKind[kind][number]?.id ?? item.trackId,
      start: (item.timelineRange.start + delta) as Flicks,
    };
  });
}
