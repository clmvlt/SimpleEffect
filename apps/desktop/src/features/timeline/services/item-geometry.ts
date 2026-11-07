import type { ItemEntity } from '@se/protocol';

import type { TimeSpan } from './in-out';
import { timeToX, type Viewport } from './viewport';
import { itemEnd } from './visible-items';

/** Pixels rendered beyond each edge of the lanes, so scrolling never shows a gap. */
export const OVERSCAN_PIXELS = 200;

/** Where a clip is drawn: only its visible part, so a long clip never makes a huge element. */
export interface ItemBox {
  /** Left edge in the lanes, in pixels (≥ −overscan). */
  readonly left: number;
  /** Width drawn, in pixels (at least 1 so very short clips stay visible). */
  readonly width: number;
  /** Pixels of the clip cut off left of `left` (to offset its pictures). */
  readonly offsetPixels: number;
}

/** The drawn box of `item` in `viewport`. */
export function itemBox(item: ItemEntity, viewport: Viewport): ItemBox {
  const start = timeToX(item.timelineRange.start, viewport);
  const end = timeToX(itemEnd(item), viewport);
  const left = Math.max(start, -OVERSCAN_PIXELS);
  const right = Math.min(end, viewport.width + OVERSCAN_PIXELS);
  return { left, width: Math.max(1, right - left), offsetPixels: left - start };
}

/** Where a span (the in/out points) is drawn in the lanes or the ruler. */
export function spanBox(
  span: TimeSpan,
  viewport: Viewport,
): { readonly left: number; readonly width: number } {
  const left = timeToX(span.start, viewport);
  return { left, width: timeToX(span.end, viewport) - left };
}
