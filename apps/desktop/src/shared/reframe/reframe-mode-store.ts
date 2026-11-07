import type { ItemId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

interface ReframeModeState {
  /** The clip whose kept area the viewer is moving, or `null`. */
  readonly itemId: ItemId | null;
}

/**
 * The viewer's reframing mode (prompt 16): while set, the viewer shows the whole picture of a
 * clip that fills the frame around it and dragging moves the part kept. The inspector's
 * « Recadrer dans le viewer » and a double click on the clip in the viewer start it.
 */
export const reframeModeStore = createStore<ReframeModeState>()(() => ({ itemId: null }));

/** Starts reframing `itemId` in the viewer. */
export function startReframing(itemId: ItemId): void {
  reframeModeStore.setState({ itemId });
}

/** Leaves the reframing mode. */
export function stopReframing(): void {
  reframeModeStore.setState({ itemId: null });
}

/** The clip being reframed, or `null`. */
export function useReframingItem(): ItemId | null {
  return useStore(reframeModeStore, (state) => state.itemId);
}
