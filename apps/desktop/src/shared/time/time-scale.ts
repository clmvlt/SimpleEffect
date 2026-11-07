import type { Flicks } from '@se/protocol';

import { clampFlicks, FLICKS_PER_SECOND } from './flicks';

/**
 * Time ↔ pixels for a zoom level given in pixels per second (the timeline's scale). Pixels are a
 * display boundary: positions computed back from pixels are rounded to whole flicks.
 */

/** Horizontal pixels of `time` at `pixelsPerSecond`. */
export function flicksToPixels(time: Flicks, pixelsPerSecond: number): number {
  return (time / FLICKS_PER_SECOND) * pixelsPerSecond;
}

/** The time `pixels` stand for at `pixelsPerSecond`, rounded to the nearest flick. */
export function pixelsToFlicks(pixels: number, pixelsPerSecond: number): Flicks {
  return clampFlicks((pixels / pixelsPerSecond) * FLICKS_PER_SECOND);
}
