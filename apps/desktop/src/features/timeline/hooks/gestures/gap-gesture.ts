import type { Flicks } from '@se/protocol';

import { gapDuration, gapResizePreview, type Gap } from '../../services/gaps';
import { concernedTrackIds } from '../../services/ripple-preview';
import { clearGesturePreview, setGesturePreview } from '../../store/gesture-store';
import { clearSelection, selectGap } from '../../store/selection-store';
import {
  frameDelta,
  snapEdges,
  type GestureContext,
  type LaneGesture,
  type LanePoint,
} from './gesture-context';

/**
 * Dragging the handle at the end of `gap`: right lengthens it, left shortens it (down to closing
 * it); what follows moves on its track and the tracks of items linked to it. One `resizeGap`
 * command on release; the resized gap stays selected.
 */
export function startGapGesture(
  context: () => GestureContext,
  gap: Gap,
  origin: LanePoint,
): LaneGesture {
  const initial = gapDuration(gap);
  let duration = initial;

  const move = (point: LanePoint) => {
    const current = context();
    const { tracks, itemsByTrack } = current.model;
    const wanted = Math.max(0, initial + frameDelta(current, point.x - origin.x)) as Flicks;
    const concerned = concernedTrackIds(tracks, itemsByTrack, new Set([gap.trackId]), gap.end);
    const following = [...concerned].flatMap((id) =>
      (itemsByTrack.get(id) ?? []).filter((item) => item.timelineRange.start >= gap.end),
    );
    const snap = snapEdges(current, [(gap.start + wanted) as Flicks], following);
    duration = Math.max(0, snap === null ? wanted : snap.point - gap.start) as Flicks;
    const placements = gapResizePreview(gap, duration, tracks, itemsByTrack);
    const resized = { ...gap, end: (gap.start + duration) as Flicks };
    setGesturePreview({ placements, ghosts: [], snapTime: snap?.point ?? null, gap: resized });
  };

  const finish = async (point: LanePoint) => {
    move(point);
    if (duration !== initial) {
      const done = await context().commands.resizeGap(gap, duration);
      if (done !== null && duration > 0) {
        selectGap({ ...gap, end: (gap.start + duration) as Flicks });
      } else if (done !== null) {
        clearSelection();
      }
    }
    clearGesturePreview();
  };

  return { move, finish, cancel: clearGesturePreview };
}
