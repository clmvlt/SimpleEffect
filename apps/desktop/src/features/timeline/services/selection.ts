import type { Flicks, ItemEntity, ItemId } from '@se/protocol';

import { itemEnd } from './visible-items';

/** How a click changes the selection: replace it, add to it, or toggle the clicked items. */
export type SelectionMode = 'replace' | 'add' | 'toggle';

/** The selection mode of a click: Maj adds, Ctrl/⌘ toggles, a plain click replaces. */
export function selectionModeOf(
  event: { readonly shiftKey: boolean; readonly ctrlKey: boolean; readonly metaKey: boolean },
  apple: boolean,
): SelectionMode {
  if (apple ? event.metaKey : event.ctrlKey) return 'toggle';
  return event.shiftKey ? 'add' : 'replace';
}

/**
 * The selection right when an item is pressed: unchanged when `clicked` is already selected (a
 * drag then moves the whole selection; the click's own meaning waits for the release),
 * otherwise `clicked` replaces or joins it.
 */
export function selectionOnPress(
  current: readonly ItemId[],
  clicked: readonly ItemId[],
  mode: SelectionMode,
): ItemId[] {
  const selected = new Set(current);
  if (clicked.every((id) => selected.has(id))) return [...current];
  return nextSelection(current, clicked, mode === 'replace' ? 'replace' : 'add');
}

/** `current` changed by clicking `clicked` (the item and its linked partners) with `mode`. */
export function nextSelection(
  current: readonly ItemId[],
  clicked: readonly ItemId[],
  mode: SelectionMode,
): ItemId[] {
  if (mode === 'replace') return [...clicked];
  const selected = new Set(current);
  const allSelected = clicked.every((id) => selected.has(id));
  for (const id of clicked) {
    if (mode === 'toggle' && allSelected) selected.delete(id);
    else selected.add(id);
  }
  return [...selected];
}

/**
 * The item and the items linked to it (a clip's picture and sound select together). Alt selects
 * the item alone.
 */
export function linkedItems(
  item: ItemEntity,
  items: Readonly<Record<ItemId, ItemEntity>>,
  sequenceItemIds: readonly ItemId[],
  alone: boolean,
): ItemId[] {
  if (alone || item.linkGroup === null) return [item.id];
  return sequenceItemIds.filter((id) => items[id]?.linkGroup === item.linkGroup);
}

/** A rectangle drawn over the lanes: a time span and the rows it crosses. */
export interface Marquee {
  readonly start: Flicks;
  readonly end: Flicks;
  /** Ids of the tracks whose rows the rectangle crosses. */
  readonly trackIds: ReadonlySet<string>;
}

/** The items a marquee touches (any overlap with its time span, on a crossed track). */
export function itemsInMarquee(marquee: Marquee, items: Iterable<ItemEntity>): ItemId[] {
  const touched: ItemId[] = [];
  for (const item of items) {
    if (
      marquee.trackIds.has(item.trackId) &&
      item.timelineRange.start < marquee.end &&
      itemEnd(item) > marquee.start
    ) {
      touched.push(item.id);
    }
  }
  return touched;
}
