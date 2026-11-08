import type { Flicks, Task, TaskId } from '@se/protocol';

import { rangeEnd } from '../../../shared/ai-tasks';
import { OVERSCAN_PIXELS } from './item-geometry';
import { allRows, type TrackLayout } from './track-layout';
import { timeToX, type Viewport } from './viewport';

/** Where an active AI task's scope is shaded on the lanes. */
export interface AiScopeBox {
  readonly key: string;
  readonly taskId: TaskId;
  readonly left: number;
  readonly width: number;
  readonly top: number;
  readonly height: number;
  /** The box carrying the task's label (the topmost of its rows). */
  readonly labelled: boolean;
}

/**
 * Horizontal extent of a scope (the whole sequence when it has no range), clamped to the view and
 * the overscan items are drawn in (clamped edges never show; widths stay small when zoomed in).
 */
function extent(task: Task, viewport: Viewport): { left: number; width: number } | null {
  const range = task.scope.range;
  const start = range === null ? 0 : range.start;
  const left = timeToX(start as Flicks, viewport);
  const right =
    range === null
      ? viewport.width + OVERSCAN_PIXELS
      : timeToX(rangeEnd(range) as Flicks, viewport);
  const clampedLeft = Math.max(-OVERSCAN_PIXELS, left);
  const clampedRight = Math.min(viewport.width + OVERSCAN_PIXELS, right);
  return clampedRight > clampedLeft
    ? { left: clampedLeft, width: clampedRight - clampedLeft }
    : null;
}

/**
 * The boxes shading the scopes of `tasks` (the active tasks of the sequence shown): across every
 * row when a scope names no tracks, else one box per row of its tracks, the topmost labelled.
 */
export function aiScopeBoxes(
  tasks: readonly Task[],
  layout: TrackLayout,
  viewport: Viewport,
): AiScopeBox[] {
  const rows = allRows(layout);
  return tasks.flatMap((task) => {
    const span = extent(task, viewport);
    if (span === null) return [];
    if (task.scope.tracks === null) {
      return [
        { key: task.id, taskId: task.id, ...span, top: 0, height: layout.height, labelled: true },
      ];
    }
    const tracks = task.scope.tracks;
    return rows
      .filter((row) => tracks.includes(row.track.id))
      .sort((left, right) => left.top - right.top)
      .map((row, index) => ({
        key: `${task.id}:${row.track.id}`,
        taskId: task.id,
        ...span,
        top: row.top,
        height: row.height + row.laneHeight,
        labelled: index === 0,
      }));
  });
}
