import type { SequenceEntity } from '@se/protocol';
import { useStore } from 'zustand';

import { useProjectSelector } from '../../../shared/project-store';
import { timelineViewStore } from '../store/timeline-view-store';

/**
 * The sequence the editor shows: the one chosen in the timeline tabs, or the project's first
 * sequence; `null` without a project or sequence.
 */
export function useActiveSequence(): SequenceEntity | null {
  const activeId = useStore(timelineViewStore, (state) => state.activeSequenceId);
  const sequences = useProjectSelector((state) => state.mirror?.sequences);
  const firstId = useProjectSelector((state) => state.mirror?.sequenceOrder[0]);
  if (sequences === undefined) return null;
  const active = activeId === null ? undefined : sequences[activeId];
  return active ?? (firstId === undefined ? null : (sequences[firstId] ?? null));
}
