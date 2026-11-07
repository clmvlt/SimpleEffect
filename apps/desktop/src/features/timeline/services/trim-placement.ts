import type {
  Asset,
  Flicks,
  ItemEdge,
  ItemEntity,
  ItemId,
  ItemTrim,
  TrackEntity,
  TrackId,
} from '@se/protocol';

import { placedAt, type PreviewPlacement } from './preview';
import { concernedTrackIds, shiftedPlacements } from './ripple-preview';
import { itemEnd } from './visible-items';

/** Where an edge of `item` is. */
export function edgeTime(item: ItemEntity, edge: ItemEdge): Flicks {
  return edge === 'start' ? item.timelineRange.start : itemEnd(item);
}

/**
 * The items an edge drag trims: the item and its linked partners whose same edge sits at the
 * same instant (a clip's picture and sound), or the item alone with Alt.
 */
export function trimGroup(
  item: ItemEntity,
  edge: ItemEdge,
  sequenceItems: readonly ItemEntity[],
  alone: boolean,
): ItemEntity[] {
  if (alone || item.linkGroup === null) return [item];
  const at = edgeTime(item, edge);
  return sequenceItems.filter(
    (other) =>
      other.id === item.id || (other.linkGroup === item.linkGroup && edgeTime(other, edge) === at),
  );
}

/** Where a trimmed edge may go. */
export interface TrimLimits {
  readonly min: Flicks;
  readonly max: Flicks;
}

/** What limits a trim besides the items themselves. */
export interface TrimContext {
  readonly itemsByTrack: ReadonlyMap<TrackId, readonly ItemEntity[]>;
  readonly assets: Readonly<Record<string, Asset>>;
  /** One frame of the sequence: the shortest an item may become. */
  readonly frame: Flicks;
  /** A ripple trim moves what follows instead of stopping at the neighbours. */
  readonly ripple: boolean;
}

/** The media a clip may play around its current range: `[before, after]` in sequence time. */
function sourceRoom(item: ItemEntity, assets: Readonly<Record<string, Asset>>) {
  if (item.kind.kind !== 'mediaClip') return { before: -Infinity, after: Infinity };
  const { sourceStart, speed, assetId } = item.kind;
  const duration = assets[assetId]?.mediaInfo?.duration ?? null;
  const before = item.timelineRange.start - sourceStart / speed;
  const after =
    duration === null ? Infinity : item.timelineRange.start + (duration - sourceStart) / speed;
  return { before, after };
}

/** The neighbours' edges around `item` on its track. */
function neighbours(item: ItemEntity, trackItems: readonly ItemEntity[]) {
  const index = trackItems.findIndex((other) => other.id === item.id);
  const previous = trackItems[index - 1];
  const next = trackItems[index + 1];
  return {
    previousEnd: previous === undefined ? 0 : itemEnd(previous),
    nextStart: next === undefined ? Infinity : next.timelineRange.start,
  };
}

/**
 * The range the edge of `items` may move in: each item keeps at least one frame and plays only
 * media it has; without ripple it stops at its neighbours and at zero.
 */
export function trimLimits(
  items: readonly ItemEntity[],
  edge: ItemEdge,
  context: TrimContext,
): TrimLimits {
  let min = -Infinity;
  let max = Infinity;
  for (const item of items) {
    const room = sourceRoom(item, context.assets);
    const around = neighbours(item, context.itemsByTrack.get(item.trackId) ?? []);
    if (edge === 'start') {
      max = Math.min(max, itemEnd(item) - context.frame);
      min = Math.max(min, room.before, context.ripple ? -Infinity : around.previousEnd);
      if (!context.ripple) min = Math.max(min, 0);
    } else {
      min = Math.max(min, item.timelineRange.start + context.frame);
      max = Math.min(max, room.after, context.ripple ? Infinity : around.nextStart);
    }
  }
  return { min: Math.ceil(min) as Flicks, max: Math.floor(max) as Flicks };
}

/** `to` kept inside `limits` (the edge stays put when the limits cross). */
export function clampTrim(to: Flicks, limits: TrimLimits, current: Flicks): Flicks {
  if (limits.min > limits.max) return current;
  return Math.min(limits.max, Math.max(limits.min, to)) as Flicks;
}

/** The `trimItems` request moving the edge of `items` to `to`. */
export function trimRequests(items: readonly ItemEntity[], edge: ItemEdge, to: Flicks): ItemTrim[] {
  return items.map((item) => ({ itemId: item.id, edge, to }));
}

/**
 * How a trim of `items` to `to` shows: the items' new extent, and with ripple what follows on the
 * concerned tracks moved by the change (a ripple trim of a start keeps the item where it starts).
 */
export function trimPreview(
  items: readonly ItemEntity[],
  edge: ItemEdge,
  to: Flicks,
  ripple: boolean,
  tracks: readonly TrackEntity[],
  itemsByTrack: ReadonlyMap<TrackId, readonly ItemEntity[]>,
): Map<ItemId, PreviewPlacement> {
  const first = items[0];
  if (first === undefined) return new Map();
  const from = edgeTime(first, edge);
  const change = (to - from) as Flicks;
  const shift = (edge === 'end' ? to - from : from - to) as Flicks;
  let placements = new Map<ItemId, PreviewPlacement>();
  if (ripple) {
    const seeds = new Set(items.map((item) => item.trackId));
    const concerned = concernedTrackIds(tracks, itemsByTrack, seeds, from);
    const trimmed = new Set(items.map((item) => item.id));
    placements = shiftedPlacements(itemsByTrack, concerned, from, shift, trimmed);
  }
  for (const item of items) {
    const { start } = item.timelineRange;
    const end = itemEnd(item);
    const [newStart, newEnd] =
      edge === 'end' ? [start, to] : ripple ? [start, end - change] : [to, end];
    placements.set(item.id, {
      ...placedAt(item, item.trackId, newStart),
      duration: (newEnd - newStart) as Flicks,
    });
  }
  return placements;
}
