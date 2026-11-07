import type { Flicks, ProjectId, SequenceId, TrackId } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

import { ZERO_FLICKS } from '../../../shared/time';
import { NO_IN_OUT, type InOutPoints } from '../services/in-out';
import { clampTrackHeight } from '../services/track-height';

/** How one sequence is shown: zoom (`null` = fit when first shown), scroll and in/out points. */
export interface SequenceView extends InOutPoints {
  readonly pixelsPerSecond: number | null;
  readonly scrollTime: Flicks;
}

/**
 * UI state of the timeline (machine preferences, never project data: `timeline-persistence.ts`
 * keeps it per project in `localStorage`). The playhead has its own store (frequent updates).
 */
export interface TimelineViewState {
  /** The project this state belongs to. */
  readonly projectId: ProjectId | null;
  /** The sequence chosen in the tabs (`null`: the first one). */
  readonly activeSequenceId: SequenceId | null;
  readonly sequences: Readonly<Record<SequenceId, SequenceView>>;
  /** Heights of the tracks the user resized. */
  readonly trackHeights: Readonly<Record<TrackId, number>>;
  /** Whether the playhead snaps to edit points, markers and in/out points while dragged. */
  readonly snapping: boolean;
  /** Width of the lanes in pixels (measured, not saved). */
  readonly lanesWidth: number;
}

/** The view of a sequence never shown before. */
export const NEW_SEQUENCE_VIEW: SequenceView = {
  pixelsPerSecond: null,
  scrollTime: ZERO_FLICKS,
  ...NO_IN_OUT,
};

/** The timeline view store. */
export const timelineViewStore = createStore<TimelineViewState>()(() => ({
  projectId: null,
  activeSequenceId: null,
  sequences: {},
  trackHeights: {},
  snapping: true,
  lanesWidth: 0,
}));

/** The view of `sequenceId` in `state`. */
export function sequenceViewOf(state: TimelineViewState, sequenceId: SequenceId): SequenceView {
  return state.sequences[sequenceId] ?? NEW_SEQUENCE_VIEW;
}

/** Changes part of the view of a sequence. */
export function updateSequenceView(sequenceId: SequenceId, changes: Partial<SequenceView>): void {
  timelineViewStore.setState((state) => ({
    sequences: {
      ...state.sequences,
      [sequenceId]: { ...sequenceViewOf(state, sequenceId), ...changes },
    },
  }));
}

/** Shows `sequenceId` in the timeline (and the viewer). */
export function setActiveSequence(sequenceId: SequenceId): void {
  timelineViewStore.setState({ activeSequenceId: sequenceId });
}

/** Sets the heights of `trackIds` (clamped). */
export function setTrackHeights(trackIds: readonly TrackId[], height: number): void {
  const clamped = clampTrackHeight(height);
  timelineViewStore.setState((state) => ({
    trackHeights: {
      ...state.trackHeights,
      ...Object.fromEntries(trackIds.map((id) => [id, clamped])),
    },
  }));
}

/** Turns playhead snapping on or off. */
export function setSnapping(snapping: boolean): void {
  timelineViewStore.setState({ snapping });
}

/** Records the measured width of the lanes. */
export function setLanesWidth(width: number): void {
  if (width !== timelineViewStore.getState().lanesWidth) {
    timelineViewStore.setState({ lanesWidth: width });
  }
}
