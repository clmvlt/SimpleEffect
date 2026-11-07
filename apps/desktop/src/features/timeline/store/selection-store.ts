import type { ItemId, TrackId, TransitionId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { Gap } from '../services/gaps';

/**
 * What is selected in the timeline: items (shared with other features through the timeline's
 * `index.ts`: inspector, AI scope) or one gap, and the active track (where pastes go). May hold
 * ids of items that were deleted since: readers skip them.
 */
export interface SelectionState {
  readonly itemIds: readonly ItemId[];
  /** The selected gap (selecting items clears it, and the other way round). */
  readonly gap: Gap | null;
  /** The selected transition (selecting items or a gap clears it, and the other way round). */
  readonly transitionId: TransitionId | null;
  /** The track last clicked: pastes land on it. */
  readonly activeTrackId: TrackId | null;
}

const EMPTY: readonly ItemId[] = [];

/** The selection store. */
export const selectionStore = createStore<SelectionState>()(() => ({
  itemIds: EMPTY,
  gap: null,
  transitionId: null,
  activeTrackId: null,
}));

/** Replaces the selection by items. */
export function selectItems(itemIds: readonly ItemId[]): void {
  selectionStore.setState({
    itemIds: itemIds.length === 0 ? EMPTY : itemIds,
    gap: null,
    transitionId: null,
  });
}

/** Selects one gap (and makes its track the active one). */
export function selectGap(gap: Gap): void {
  selectionStore.setState({ itemIds: EMPTY, gap, transitionId: null, activeTrackId: gap.trackId });
}

/** Selects one transition (`Suppr` removes it). */
export function selectTransition(transitionId: TransitionId, trackId: TrackId): void {
  selectionStore.setState({ itemIds: EMPTY, gap: null, transitionId, activeTrackId: trackId });
}

/** Makes `trackId` the track pastes go to. */
export function setActiveTrack(trackId: TrackId): void {
  if (selectionStore.getState().activeTrackId === trackId) return;
  selectionStore.setState({ activeTrackId: trackId });
}

/** Empties the selection (items and gap). */
export function clearSelection(): void {
  selectionStore.setState({ itemIds: EMPTY, gap: null, transitionId: null });
}

/** The selected item ids (stable reference until the selection changes). */
export function useSelectedItemIds(): readonly ItemId[] {
  return useStore(selectionStore, (state) => state.itemIds);
}

/** The selected transition, if any. */
export function useSelectedTransition(): TransitionId | null {
  return useStore(selectionStore, (state) => state.transitionId);
}

/** The selected gap, if any. */
export function useSelectedGap(): Gap | null {
  return useStore(selectionStore, (state) => state.gap);
}
