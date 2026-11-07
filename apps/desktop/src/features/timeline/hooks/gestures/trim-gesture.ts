import type { Flicks, ItemEdge, ItemEntity } from '@se/protocol';

import { frameDuration } from '../../../../shared/time';
import {
  clampTrim,
  edgeTime,
  trimLimits,
  trimPreview,
  trimRequests,
} from '../../services/trim-placement';
import { clearGesturePreview, setGesturePreview } from '../../store/gesture-store';
import {
  frameDelta,
  lockedTrackIds,
  snapEdges,
  type GestureContext,
  type GestureModifiers,
  type LaneGesture,
  type LanePoint,
} from './gesture-context';

/**
 * Dragging the `edge` of `items` (an item and the linked partners whose edge sits at the same
 * instant) from `origin`: frame by frame, snapping, within the media and one frame from the other
 * edge, stopping at neighbours unless Ctrl/⌘ (ripple trim: what follows moves). One `trimItems`
 * command on release. `null` on a locked track.
 */
export function startTrimGesture(
  context: () => GestureContext,
  items: readonly ItemEntity[],
  edge: ItemEdge,
  origin: LanePoint,
): LaneGesture | null {
  const first = items[0];
  const locked = lockedTrackIds(context());
  if (first === undefined || items.some((item) => locked.has(item.trackId))) return null;
  const from = edgeTime(first, edge);
  let to = from;

  const move = (point: LanePoint, modifiers: GestureModifiers) => {
    const current = context();
    const { model } = current;
    const limits = trimLimits(items, edge, {
      itemsByTrack: model.itemsByTrack,
      assets: model.assets,
      frame: frameDuration(model.sequence.format.frameRate) as Flicks,
      ripple: modifiers.insert,
    });
    const wanted = (from + frameDelta(current, point.x - origin.x)) as Flicks;
    const snap = snapEdges(current, [wanted], items);
    to = clampTrim(snap === null ? wanted : snap.point, limits, from);
    const placements = trimPreview(
      items,
      edge,
      to,
      modifiers.insert,
      model.tracks,
      model.itemsByTrack,
    );
    const snapTime = snap !== null && snap.point === to ? to : null;
    setGesturePreview({ placements, ghosts: [], snapTime });
  };

  const finish = async (point: LanePoint, modifiers: GestureModifiers) => {
    move(point, modifiers);
    if (to !== from) {
      await context().commands.trimItems(trimRequests(items, edge, to), modifiers.insert);
    }
    clearGesturePreview();
  };

  return { move, finish, cancel: clearGesturePreview };
}
