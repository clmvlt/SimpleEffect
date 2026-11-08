import type { SequenceId, TimeRange } from '@se/protocol';
import { createStore } from 'zustand/vanilla';
import { useStore } from 'zustand';

/** Cuts shown on the timeline before they are made (prompt 22): temporary, never project data. */
export interface CutPreview {
  /** The sequence they belong to, or `null` when nothing is previewed. */
  readonly sequenceId: SequenceId | null;
  /** The ranges a cut would remove, sorted. */
  readonly ranges: readonly TimeRange[];
}

const NOTHING: CutPreview = { sequenceId: null, ranges: [] };

/** The previewed cuts (the silence removal panel writes them, the lanes draw them). */
export const cutPreviewStore = createStore<CutPreview>()(() => NOTHING);

/** Shows `ranges` of `sequenceId` as cuts to come. */
export function showCutPreview(sequenceId: SequenceId, ranges: readonly TimeRange[]): void {
  cutPreviewStore.setState({ sequenceId, ranges });
}

/** Removes the previewed cuts. */
export function clearCutPreview(): void {
  cutPreviewStore.setState(NOTHING);
}

/** The previewed cuts of `sequenceId` (empty for another sequence). */
export function useCutPreview(sequenceId: SequenceId): readonly TimeRange[] {
  return useStore(cutPreviewStore, (state) =>
    state.sequenceId === sequenceId ? state.ranges : NOTHING.ranges,
  );
}
