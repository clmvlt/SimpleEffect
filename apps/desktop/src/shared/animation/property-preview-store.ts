import type { ItemAnimation, ItemEntity, ItemId, PropertyAnimation } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { withAnimation } from './property-access';

/**
 * Ephemeral values shown while a continuous edit runs (a gizmo drag, a `ScrubInput` drag, typing
 * in the viewer's text field): the viewer draws items with them, and they are cleared once the
 * one command sent on release resolved or failed. Never document state (golden rule 4).
 */
export interface PropertyPreviewState {
  /** Property values by item, applied in order over the mirror's item. */
  readonly animations: Readonly<Record<ItemId, readonly PropertyAnimation[]>>;
  /** Text contents being typed, by item. */
  readonly contents: Readonly<Record<ItemId, string>>;
}

const EMPTY: PropertyPreviewState = { animations: {}, contents: {} };

/** The preview store. */
export const propertyPreviewStore = createStore<PropertyPreviewState>()(() => EMPTY);

/** Shows `changes` instead of the document's values (replacing earlier previews of the same properties). */
export function previewAnimations(changes: readonly ItemAnimation[]): void {
  const animations: Record<ItemId, PropertyAnimation[]> = {
    ...propertyPreviewStore.getState().animations,
  } as Record<ItemId, PropertyAnimation[]>;
  for (const { itemId, animation } of changes) {
    const others = (animations[itemId] ?? []).filter(
      (previous) => previous.property !== animation.property,
    );
    animations[itemId] = [...others, animation];
  }
  propertyPreviewStore.setState({ animations });
}

/** Shows `content` as the text of `itemId` while it is typed. */
export function previewContent(itemId: ItemId, content: string): void {
  const contents = { ...propertyPreviewStore.getState().contents, [itemId]: content };
  propertyPreviewStore.setState({ contents });
}

/** Drops every preview (the document's values show again). */
export function clearPropertyPreview(): void {
  if (propertyPreviewStore.getState() !== EMPTY) propertyPreviewStore.setState(EMPTY);
}

/** `item` as previewed: its previewed property values and content applied (`item` itself when none). */
export function previewedItem(item: ItemEntity, state: PropertyPreviewState): ItemEntity {
  const animations = state.animations[item.id];
  const content = state.contents[item.id];
  if (animations === undefined && content === undefined) return item;
  let kind = item.kind;
  for (const animation of animations ?? []) kind = withAnimation(kind, animation);
  if (content !== undefined && kind.kind === 'text') kind = { ...kind, content };
  return { ...item, kind };
}

/** The preview state (re-renders on every change: for the viewer's scene). */
export function usePropertyPreview(): PropertyPreviewState {
  return useStore(propertyPreviewStore);
}
