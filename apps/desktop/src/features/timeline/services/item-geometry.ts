import type { Flicks, ItemEntity } from '@se/protocol';

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
  /** Whether the clip's real start is drawn (its trim handle can show). */
  readonly startVisible: boolean;
  /** Whether the clip's real end is drawn. */
  readonly endVisible: boolean;
}

/** The drawn box of `[start, end)` in `viewport`, cut to the visible part plus the overscan. */
export function rangeBox(start: Flicks, end: Flicks, viewport: Viewport): ItemBox {
  const startX = timeToX(start, viewport);
  const endX = timeToX(end, viewport);
  const left = Math.max(startX, -OVERSCAN_PIXELS);
  const right = Math.min(endX, viewport.width + OVERSCAN_PIXELS);
  return {
    left,
    width: Math.max(1, right - left),
    offsetPixels: left - startX,
    startVisible: left === startX,
    endVisible: right === endX,
  };
}

/** The drawn box of `item` in `viewport`. */
export function itemBox(item: ItemEntity, viewport: Viewport): ItemBox {
  return rangeBox(item.timelineRange.start, itemEnd(item), viewport);
}

/** Where a span (the in/out points) is drawn in the lanes or the ruler. */
export function spanBox(
  span: TimeSpan,
  viewport: Viewport,
): { readonly left: number; readonly width: number } {
  const left = timeToX(span.start, viewport);
  return { left, width: timeToX(span.end, viewport) - left };
}
