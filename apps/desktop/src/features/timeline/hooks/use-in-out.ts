import type { Flicks, SequenceEntity, SequenceId } from '@se/protocol';
import { useStore } from 'zustand';

import { clampFlicks, frameDuration } from '../../../shared/time';
import {
  inOutSpan,
  NO_IN_OUT,
  pointsBetween,
  withInPoint,
  withOutPoint,
  type InOutPoints,
  type TimeSpan,
} from '../services/in-out';
import {
  sequenceViewOf,
  timelineViewStore,
  updateSequenceView,
} from '../store/timeline-view-store';

/** The in and out points of `sequenceId` (stable values, re-rendering when they change). */
export function useInOutPoints(sequenceId: SequenceId): InOutPoints {
  const inPoint = useStore(timelineViewStore, (state) => sequenceViewOf(state, sequenceId).inPoint);
  const outPoint = useStore(
    timelineViewStore,
    (state) => sequenceViewOf(state, sequenceId).outPoint,
  );
  return { inPoint, outPoint };
}

/**
 * The span marked by the in and out points of `sequenceId` (to the content end when only the in
 * point is set), or `null`: the scope of AI requests (prompt 20).
 */
export function useInOutSpan(sequenceId: SequenceId, contentEnd: Flicks): TimeSpan | null {
  return inOutSpan(useInOutPoints(sequenceId), contentEnd);
}

/** Setting and clearing the in and out points of a sequence. */
export interface InOutControls {
  /** In point at the frame starting at `time`. */
  readonly markIn: (time: Flicks) => void;
  /** Out point after the frame starting at `time` (that frame is included). */
  readonly markOut: (time: Flicks) => void;
  /** Both points from a span drawn on the ruler. */
  readonly markSpan: (from: Flicks, to: Flicks) => void;
  readonly clear: () => void;
}

/** In/out controls of `sequence`. */
export function inOutControls(sequence: SequenceEntity): InOutControls {
  const points = () => sequenceViewOf(timelineViewStore.getState(), sequence.id);
  const frame = frameDuration(sequence.format.frameRate);
  return {
    markIn: (time) => {
      updateSequenceView(sequence.id, withInPoint(points(), time));
    },
    markOut: (time) => {
      updateSequenceView(sequence.id, withOutPoint(points(), clampFlicks(time + frame)));
    },
    markSpan: (from, to) => {
      updateSequenceView(sequence.id, pointsBetween(from, to));
    },
    clear: () => {
      updateSequenceView(sequence.id, NO_IN_OUT);
    },
  };
}
