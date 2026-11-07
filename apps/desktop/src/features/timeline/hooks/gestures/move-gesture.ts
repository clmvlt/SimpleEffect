import type { Flicks, ItemEntity, ItemId, PlacementMode } from '@se/protocol';

import {
  clampTimeDelta,
  clampTrackOffset,
  dragTrackOffset,
  movedPlacements,
  type Placement,
} from '../../services/move-placement';
import { placedAt, type GesturePreview, type PreviewPlacement } from '../../services/preview';
import { concernedTrackIds, shiftedPlacements } from '../../services/ripple-preview';
import { rowAt } from '../../services/track-layout';
import { itemEnd } from '../../services/visible-items';
import { clearGesturePreview, setGesturePreview } from '../../store/gesture-store';
import { selectItems } from '../../store/selection-store';
import {
  frameDelta,
  lockedTrackIds,
  snapEdges,
  type GestureContext,
  type GestureModifiers,
  type LaneGesture,
  type LanePoint,
} from './gesture-context';

interface MoveState {
  readonly placements: readonly Placement[];
  readonly moved: boolean;
}

/** Items starting at or after the placed block on its concerned tracks, drawn pushed by it. */
function insertShift(
  context: GestureContext,
  placements: readonly Placement[],
  items: readonly ItemEntity[],
): Map<ItemId, PreviewPlacement> {
  const durations = new Map(items.map((item) => [item.id, item.timelineRange.duration]));
  const start = Math.min(...placements.map((placement) => placement.start)) as Flicks;
  const end = Math.max(...placements.map((p) => p.start + (durations.get(p.itemId) ?? 0)));
  const { tracks, itemsByTrack } = context.model;
  const seeds = new Set(placements.map((placement) => placement.trackId));
  const concerned = concernedTrackIds(tracks, itemsByTrack, seeds, start);
  const dragged = new Set(items.map((item) => item.id));
  return shiftedPlacements(itemsByTrack, concerned, start, (end - start) as Flicks, dragged);
}

/** What a move shows: moved items (or their copies with Alt) and, in insert mode, what is pushed. */
function movePreview(
  context: GestureContext,
  items: readonly ItemEntity[],
  placements: readonly Placement[],
  modifiers: GestureModifiers,
  snapTime: Flicks | null,
): GesturePreview {
  const byId = new Map(items.map((item) => [item.id, item]));
  const shifted = modifiers.insert ? insertShift(context, placements, items) : new Map();
  if (modifiers.alt) {
    const ghosts = placements.map((placement) => ({
      key: `copy-${placement.itemId}`,
      row: { kind: 'track' as const, trackId: placement.trackId },
      start: placement.start,
      duration: byId.get(placement.itemId)?.timelineRange.duration ?? (0 as Flicks),
      source: { kind: 'item' as const, itemId: placement.itemId },
    }));
    return { placements: shifted, ghosts, snapTime };
  }
  for (const placement of placements) {
    const item = byId.get(placement.itemId);
    if (item !== undefined)
      shifted.set(item.id, placedAt(item, placement.trackId, placement.start));
  }
  return { placements: shifted, ghosts: [], snapTime };
}

/**
 * Dragging the selected items (`items`, the grabbed one among them) from `origin`: frame by
 * frame, snapping their edges, moving them across tracks of their kind by the same count. On
 * release one `moveItems` command (Alt: `duplicateItems`; Ctrl/⌘: insert mode). `null` when an
 * item sits on a locked track (nothing may move).
 */
export function startMoveGesture(
  context: () => GestureContext,
  grabbed: ItemEntity,
  items: readonly ItemEntity[],
  origin: LanePoint,
): LaneGesture | null {
  const locked = lockedTrackIds(context());
  if (items.length === 0 || items.some((item) => locked.has(item.trackId))) return null;
  let offset = 0;
  let state: MoveState = { placements: [], moved: false };

  const move = (point: LanePoint, modifiers: GestureModifiers) => {
    const current = context();
    let delta = clampTimeDelta(items, frameDelta(current, point.x - origin.x));
    const edges = items.flatMap((item) => [
      (item.timelineRange.start + delta) as Flicks,
      (itemEnd(item) + delta) as Flicks,
    ]);
    const snap = snapEdges(current, edges, items);
    if (snap !== null) delta = clampTimeDelta(items, (delta + snap.delta) as Flicks);
    const { tracks } = current.model;
    const row = rowAt(current.layout, point.y);
    offset = dragTrackOffset(tracks, grabbed.trackId, row?.track.id ?? null, offset);
    offset = clampTrackOffset(tracks, items, offset);
    const placements = movedPlacements(items, tracks, delta, offset);
    state = { placements, moved: delta !== 0 || offset !== 0 };
    setGesturePreview(movePreview(current, items, placements, modifiers, snap?.point ?? null));
  };

  const finish = async (point: LanePoint, modifiers: GestureModifiers) => {
    move(point, modifiers);
    const { commands } = context();
    const mode: PlacementMode = modifiers.insert ? 'insert' : 'overwrite';
    if (modifiers.alt) {
      const copies = await commands.duplicateItems(state.placements, mode);
      if (copies !== null) selectItems(copies);
    } else if (state.moved) {
      await commands.moveItems(state.placements, mode);
    }
    clearGesturePreview();
  };

  return { move, finish, cancel: clearGesturePreview };
}
