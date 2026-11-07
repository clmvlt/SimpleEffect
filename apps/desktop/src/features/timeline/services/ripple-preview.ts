import type { Flicks, ItemEntity, ItemId, TrackEntity, TrackId } from '@se/protocol';

import { placedAt, type PreviewPlacement } from './preview';

/**
 * The tracks a ripple edit of `seeds` at `from` shifts, as the core computes them
 * (`concerned_tracks` in `crates/se-app/src/edit_plan/ripple.rs`, ADR-0021): the seeds, then every
 * unlocked track holding an item linked to an item starting at or after `from` on a track
 * already listed. For previews only: the core decides.
 */
export function concernedTrackIds(
  tracks: readonly TrackEntity[],
  itemsByTrack: ReadonlyMap<TrackId, readonly ItemEntity[]>,
  seeds: ReadonlySet<TrackId>,
  from: Flicks,
): Set<TrackId> {
  const concerned = new Set(seeds);
  for (;;) {
    const moving = new Set<string>();
    for (const id of concerned) {
      for (const item of itemsByTrack.get(id) ?? []) {
        if (item.timelineRange.start >= from && item.linkGroup !== null) moving.add(item.linkGroup);
      }
    }
    const added = tracks.filter(
      (track) =>
        !track.locked &&
        !concerned.has(track.id) &&
        (itemsByTrack.get(track.id) ?? []).some(
          (item) => item.linkGroup !== null && moving.has(item.linkGroup),
        ),
    );
    if (added.length === 0) return concerned;
    for (const track of added) concerned.add(track.id);
  }
}

/** Items starting at or after `from` on `trackIds` drawn `delta` later (`exclude` left out). */
export function shiftedPlacements(
  itemsByTrack: ReadonlyMap<TrackId, readonly ItemEntity[]>,
  trackIds: ReadonlySet<TrackId>,
  from: Flicks,
  delta: Flicks,
  exclude: ReadonlySet<ItemId> = new Set(),
): Map<ItemId, PreviewPlacement> {
  const placements = new Map<ItemId, PreviewPlacement>();
  if (delta === 0) return placements;
  for (const trackId of trackIds) {
    for (const item of itemsByTrack.get(trackId) ?? []) {
      if (item.timelineRange.start < from || exclude.has(item.id)) continue;
      placements.set(
        item.id,
        placedAt(item, trackId, (item.timelineRange.start + delta) as Flicks),
      );
    }
  }
  return placements;
}
