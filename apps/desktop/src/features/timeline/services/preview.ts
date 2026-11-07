import type { Flicks, ItemEntity, ItemId, TrackId } from '@se/protocol';

import type { Gap } from './gaps';
import type { ItemTone } from './item-display';

/** Where an existing item is drawn during a gesture (moved, trimmed or shifted by a ripple). */
export interface PreviewPlacement {
  readonly trackId: TrackId;
  readonly start: Flicks;
  readonly duration: Flicks;
}

/** What a ghost looks like: a copy of an item, or a part of dropped media. */
export type GhostSource =
  | { readonly kind: 'item'; readonly itemId: ItemId }
  | { readonly kind: 'media'; readonly label: string; readonly tone: ItemTone };

/** The row of a ghost: a track, or the new audio track a drop below the last track adds. */
export type GhostRow =
  { readonly kind: 'track'; readonly trackId: TrackId } | { readonly kind: 'newAudio' };

/** Where a new item would land (a drop from the media bin, an Alt+drag copy). */
export interface PreviewGhost {
  readonly key: string;
  readonly row: GhostRow;
  readonly start: Flicks;
  readonly duration: Flicks;
  readonly source: GhostSource;
}

/**
 * What a gesture shows before its command is sent (ephemeral state of the timeline, never sent
 * to the core): items drawn elsewhere, new items, and the point an edge snapped to.
 */
export interface GesturePreview {
  readonly placements: ReadonlyMap<ItemId, PreviewPlacement>;
  readonly ghosts: readonly PreviewGhost[];
  readonly snapTime: Flicks | null;
  /** The selected gap as a gap resize would leave it. */
  readonly gap?: Gap;
}

/** A preview showing only `placements`. */
export function placementsPreview(
  placements: ReadonlyMap<ItemId, PreviewPlacement>,
  snapTime: Flicks | null = null,
): GesturePreview {
  return { placements, ghosts: [], snapTime };
}

/** `item` drawn at `start` on `trackId` with its own duration. */
export function placedAt(item: ItemEntity, trackId: TrackId, start: Flicks): PreviewPlacement {
  return { trackId, start, duration: item.timelineRange.duration };
}
