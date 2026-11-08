import type { Flicks, SequenceId, TimeRange } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import { setPlayheadTime } from './playhead-store';
import {
  sequenceViewOf,
  setActiveSequence,
  timelineViewStore,
  updateSequenceView,
} from './timeline-view-store';

/** Share of the view left before a revealed range scrolled into view. */
const LEAD = 0.1;

/**
 * Shows `range` of `sequenceId` (an AI task's scope): the sequence becomes active, the range
 * becomes its in/out span, the playhead goes to its start, and the view scrolls when the start is
 * out of sight. Without a range, only the sequence is shown.
 */
export function revealTimeRange(sequenceId: SequenceId, range: TimeRange | null): void {
  setActiveSequence(sequenceId);
  if (range === null) return;
  const state = timelineViewStore.getState();
  const view = sequenceViewOf(state, sequenceId);
  const changes: { inPoint: Flicks; outPoint: Flicks; scrollTime?: Flicks } = {
    inPoint: range.start,
    outPoint: (range.start + range.duration) as Flicks,
  };
  if (view.pixelsPerSecond !== null && state.lanesWidth > 0) {
    const visible = (state.lanesWidth / view.pixelsPerSecond) * FLICKS_PER_SECOND;
    const outOfSight = range.start < view.scrollTime || range.start > view.scrollTime + visible;
    if (outOfSight) {
      changes.scrollTime = Math.max(0, Math.round(range.start - visible * LEAD)) as Flicks;
    }
  }
  updateSequenceView(sequenceId, changes);
  setPlayheadTime(sequenceId, range.start);
}
