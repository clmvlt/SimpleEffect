import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { GesturePreview } from '../services/preview';

/**
 * The ephemeral state of the gesture in progress (drag, trim, gap resize, drop): what the lanes
 * draw instead of the document until the command's patch arrives. Never sent to the core.
 */
export interface GestureState {
  readonly preview: GesturePreview | null;
}

/** The gesture store. */
export const gestureStore = createStore<GestureState>()(() => ({ preview: null }));

/** Shows `preview` over the lanes. */
export function setGesturePreview(preview: GesturePreview): void {
  gestureStore.setState({ preview });
}

/** Removes the preview (the gesture ended, its patch arrived or it failed). */
export function clearGesturePreview(): void {
  if (gestureStore.getState().preview !== null) gestureStore.setState({ preview: null });
}

/** The preview of the gesture in progress, if any. */
export function useGesturePreview(): GesturePreview | null {
  return useStore(gestureStore, (state) => state.preview);
}
