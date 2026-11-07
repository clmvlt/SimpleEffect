import type { ItemEntity, ItemId } from '@se/protocol';

import { itemsInMarquee, type SelectionMode } from './selection';
import { allRows, type TrackLayout } from './track-layout';
import { xToTime, type Viewport } from './viewport';

/** A rectangle drawn with the pointer over the lanes, in lane pixels (corners in any order). */
export interface MarqueeRect {
  readonly x0: number;
  readonly y0: number;
  readonly x1: number;
  readonly y1: number;
}

/** The rectangle normalised for drawing: left, top, width, height. */
export function marqueeBox(rect: MarqueeRect) {
  return {
    left: Math.min(rect.x0, rect.x1),
    top: Math.min(rect.y0, rect.y1),
    width: Math.abs(rect.x1 - rect.x0),
    height: Math.abs(rect.y1 - rect.y0),
  };
}

/** The items a rectangle touches: rows it crosses, times it spans. */
export function itemsInRect(
  rect: MarqueeRect,
  layout: TrackLayout,
  viewport: Viewport,
  items: Iterable<ItemEntity>,
): ItemId[] {
  const box = marqueeBox(rect);
  const trackIds = new Set(
    allRows(layout)
      .filter((row) => row.top < box.top + box.height && row.top + row.height > box.top)
      .map((row) => row.track.id),
  );
  return itemsInMarquee(
    {
      start: xToTime(box.left, viewport),
      end: xToTime(box.left + box.width, viewport),
      trackIds,
    },
    items,
  );
}

/** The selection while a rectangle is drawn from `base` with `mode`. */
export function marqueeSelection(
  base: readonly ItemId[],
  touched: readonly ItemId[],
  mode: SelectionMode,
): ItemId[] {
  if (mode === 'replace') return [...touched];
  const selected = new Set(base);
  for (const id of touched) {
    if (mode === 'toggle' && base.includes(id)) selected.delete(id);
    else selected.add(id);
  }
  return [...selected];
}
