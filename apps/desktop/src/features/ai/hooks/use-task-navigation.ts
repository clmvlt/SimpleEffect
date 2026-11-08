import type { ItemEntity, Task } from '@se/protocol';

import { useProjectMirrorReader } from '../../../shared/project-store';
import { revealTimeRange, selectItems, setPlayheadTime } from '../../timeline';
import { useTaskCommands } from './use-task-commands';

/** Ways the tasks panel points at the timeline. */
export interface TaskNavigation {
  /** Shows the task's scope: its sequence, its range as in/out span, the playhead at its start. */
  readonly revealScope: (task: Task) => void;
  /** Selects the items among `ids` that still exist, on their sequence, playhead at the first. */
  readonly selectExisting: (ids: readonly string[]) => void;
  /** « Voir les modifications »: selects what the task added or changed. */
  readonly showChanges: (task: Task) => void;
}

/** Navigation from the tasks panel to the timeline. */
export function useTaskNavigation(): TaskNavigation {
  const readMirror = useProjectMirrorReader();
  const commands = useTaskCommands();
  const selectExisting = (ids: readonly string[]) => {
    const mirror = readMirror();
    if (mirror === null) return;
    const items = ids.flatMap((id) => mirror.items[id] ?? []);
    const first = items.reduce<ItemEntity | null>(
      (earliest, item) =>
        earliest === null || item.timelineRange.start < earliest.timelineRange.start
          ? item
          : earliest,
      null,
    );
    if (first === null) return;
    const sequenceId = mirror.tracks[first.trackId]?.sequenceId;
    if (sequenceId === undefined) return;
    const onSequence = items.filter(
      (item) => mirror.tracks[item.trackId]?.sequenceId === sequenceId,
    );
    revealTimeRange(sequenceId, null);
    selectItems(onSequence.map((item) => item.id));
    setPlayheadTime(sequenceId, first.timelineRange.start);
  };
  return {
    revealScope: (task) => {
      revealTimeRange(task.scope.sequenceId, task.scope.range);
    },
    selectExisting,
    showChanges: (task) => {
      void commands.changes(task.projectId, task.id).then((ids) => {
        if (ids !== null) selectExisting(ids);
      });
    },
  };
}
