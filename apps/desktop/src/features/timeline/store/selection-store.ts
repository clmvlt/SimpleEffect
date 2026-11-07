import type { ItemId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/**
 * The selected timeline items, shared with other features through the timeline's `index.ts`
 * (inspector, AI scope). May hold ids of items that were deleted since: readers skip them.
 */
export interface SelectionState {
  readonly itemIds: readonly ItemId[];
}

const EMPTY: readonly ItemId[] = [];

/** The selection store. */
export const selectionStore = createStore<SelectionState>()(() => ({ itemIds: EMPTY }));

/** Replaces the selection. */
export function selectItems(itemIds: readonly ItemId[]): void {
  selectionStore.setState({ itemIds: itemIds.length === 0 ? EMPTY : itemIds });
}

/** Empties the selection. */
export function clearSelection(): void {
  selectionStore.setState({ itemIds: EMPTY });
}

/** The selected item ids (stable reference until the selection changes). */
export function useSelectedItemIds(): readonly ItemId[] {
  return useStore(selectionStore, (state) => state.itemIds);
}
