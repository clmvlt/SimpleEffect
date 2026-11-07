import type { AnimatableProperty, Flicks, ItemId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** A keyframe of an item's lane: its property and item-relative time. */
export interface KeyframeKey {
  readonly property: AnimatableProperty;
  readonly time: Flicks;
}

/** The keyframe lanes: which items show theirs, the selected keyframes and a drag in progress. */
export interface KeyframeLaneState {
  /** Items whose keyframe lane is unfolded under them. */
  readonly expanded: ReadonlySet<ItemId>;
  /** Selected keyframes (of one item). */
  readonly selection: { readonly itemId: ItemId; readonly keys: readonly KeyframeKey[] } | null;
  /** Shift of the selected keyframes while they are dragged (a preview, never sent). */
  readonly dragDelta: Flicks | null;
}

const EMPTY: ReadonlySet<ItemId> = new Set();

/** The keyframe lane store (view state, never project data). */
export const keyframeLaneStore = createStore<KeyframeLaneState>()(() => ({
  expanded: EMPTY,
  selection: null,
  dragDelta: null,
}));

/** Unfolds or folds the keyframe lane of `itemId`. */
export function toggleKeyframeLane(itemId: ItemId): void {
  const expanded = new Set(keyframeLaneStore.getState().expanded);
  if (!expanded.delete(itemId)) expanded.add(itemId);
  keyframeLaneStore.setState({ expanded });
}

/** Whether two keys name the same keyframe. */
export function sameKey(left: KeyframeKey, right: KeyframeKey): boolean {
  return left.property === right.property && left.time === right.time;
}

/** Selects `key` of `itemId` (`additive`: toggles it in the selection of that item). */
export function selectKeyframe(itemId: ItemId, key: KeyframeKey, additive: boolean): void {
  const current = keyframeLaneStore.getState().selection;
  if (!additive || current?.itemId !== itemId) {
    keyframeLaneStore.setState({ selection: { itemId, keys: [key] } });
    return;
  }
  const has = current.keys.some((other) => sameKey(other, key));
  const keys = has ? current.keys.filter((other) => !sameKey(other, key)) : [...current.keys, key];
  keyframeLaneStore.setState({ selection: keys.length === 0 ? null : { itemId, keys } });
}

/** Clears the keyframe selection. */
export function clearKeyframeSelection(): void {
  if (keyframeLaneStore.getState().selection !== null) {
    keyframeLaneStore.setState({ selection: null, dragDelta: null });
  }
}

/** Shows the selected keyframes shifted by `delta` while dragged (`null` ends the preview). */
export function setKeyframeDrag(delta: Flicks | null): void {
  keyframeLaneStore.setState({ dragDelta: delta });
}

/** The keyframe lane state. */
export function useKeyframeLanes(): KeyframeLaneState {
  return useStore(keyframeLaneStore);
}

/** Forgets every lane and selection (tests). */
export function resetKeyframeLanes(): void {
  keyframeLaneStore.setState({ expanded: EMPTY, selection: null, dragDelta: null });
}
