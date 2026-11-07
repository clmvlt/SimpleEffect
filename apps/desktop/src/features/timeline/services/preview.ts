import type { Animatable, Flicks, ItemEntity, ItemId, TrackId, TransitionId } from '@se/protocol';

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

/** A clip's fades or volume curve as a gesture leaves them (unset parts are unchanged). */
export interface ClipEdit {
  readonly fadeIn?: Flicks;
  readonly fadeOut?: Flicks;
  readonly volumeDb?: Animatable<number>;
}

/** A transition as a gesture draws it: a new one dragged from a cut, or one being resized. */
export interface TransitionDraft {
  readonly trackId: TrackId;
  readonly start: Flicks;
  readonly end: Flicks;
  /** The transition resized; `null` for a new one. */
  readonly transitionId: TransitionId | null;
}

/**
 * What a gesture shows before its command is sent (ephemeral state of the timeline, never sent
 * to the core): items drawn elsewhere, new items, the point an edge snapped to, and the levels or
 * transition a sound gesture is changing.
 */
export interface GesturePreview {
  readonly placements: ReadonlyMap<ItemId, PreviewPlacement>;
  readonly ghosts: readonly PreviewGhost[];
  readonly snapTime: Flicks | null;
  /** The selected gap as a gap resize would leave it. */
  readonly gap?: Gap;
  /** Fades and volume curves being dragged. */
  readonly clipEdits?: ReadonlyMap<ItemId, ClipEdit>;
  /** The transition being dragged. */
  readonly transition?: TransitionDraft;
}

/** A preview showing only a clip edit. */
export function clipEditPreview(itemId: ItemId, edit: ClipEdit): GesturePreview {
  return {
    placements: new Map(),
    ghosts: [],
    snapTime: null,
    clipEdits: new Map([[itemId, edit]]),
  };
}

/** A preview showing only a transition draft. */
export function transitionPreview(transition: TransitionDraft): GesturePreview {
  return { placements: new Map(), ghosts: [], snapTime: null, transition };
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
