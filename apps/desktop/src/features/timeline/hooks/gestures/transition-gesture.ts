import type { Flicks, TrackEntity, TransitionAlignment } from '@se/protocol';
import { transitionSpan } from '@se/engine';

import { snapToFrame } from '../../../../shared/time';
import {
  draggedTransitionDuration,
  transitionOn,
  type Cut,
  type TransitionMark,
} from '../../services/audio-edits';
import { transitionPreview } from '../../services/preview';
import { xToTime } from '../../services/viewport';
import { clearGesturePreview, setGesturePreview } from '../../store/gesture-store';
import { selectTransition } from '../../store/selection-store';
import type { GestureContext, LaneGesture, LanePoint } from './gesture-context';

/**
 * Dragging an edge of a transition: a centred one grows on both sides of its cut, others from
 * their fixed edge; whole frames. One `updateTransition` command on release (the core shortens
 * it to what the cut allows).
 */
export function startTransitionResizeGesture(
  context: () => GestureContext,
  track: TrackEntity,
  mark: TransitionMark,
  edge: 'start' | 'end',
): LaneGesture {
  const { transition } = mark;
  let duration = transition.duration;
  const move = (point: LanePoint) => {
    const current = context();
    const time = xToTime(point.x, current.viewport);
    const rate = current.model.sequence.format.frameRate;
    duration = draggedTransitionDuration(mark.span, transition.alignment, edge, time, rate);
    const span = transitionSpan({ ...transition, duration }, mark.span.cut);
    setGesturePreview(
      transitionPreview({
        trackId: track.id,
        start: span.start,
        end: span.end,
        transitionId: transition.id,
      }),
    );
  };
  const finish = async (point: LanePoint) => {
    move(point);
    if (duration !== transition.duration) {
      await context().commands.updateTransition(transition.id, { duration });
    }
    clearGesturePreview();
  };
  return { move, finish, cancel: clearGesturePreview };
}

/**
 * Dragging from a cut: a centred cross-fade as wide as twice the distance to the cut. One
 * `addTransitions` command on release; the new transition is selected.
 */
export function startTransitionCreateGesture(
  context: () => GestureContext,
  track: TrackEntity,
  cut: Cut,
): LaneGesture {
  const alignment: TransitionAlignment = 'center';
  let duration = 0 as Flicks;
  const move = (point: LanePoint) => {
    const current = context();
    const distance = Math.abs(xToTime(point.x, current.viewport) - cut.time);
    const rate = current.model.sequence.format.frameRate;
    duration = snapToFrame((2 * distance) as Flicks, rate, 'nearest');
    const half = Math.floor(duration / 2);
    setGesturePreview(
      transitionPreview({
        trackId: track.id,
        start: (cut.time - half) as Flicks,
        end: (cut.time + duration - half) as Flicks,
        transitionId: null,
      }),
    );
  };
  const finish = async (point: LanePoint) => {
    move(point);
    if (duration > 0) {
      const ids = await context().commands.addTransitions([
        transitionOn(track, cut, duration, alignment),
      ]);
      const created = ids?.[0];
      if (created !== undefined) selectTransition(created, track.id);
    }
    clearGesturePreview();
  };
  return { move, finish, cancel: clearGesturePreview };
}
