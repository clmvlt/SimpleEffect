import type { Flicks, ItemEntity, TrackKind } from '@se/protocol';

import { clampFlicks, flicksToSeconds } from '../../../shared/time';
import type { TrackRow } from './track-layout';
import { clampZoom, type Viewport, type ZoomLimits } from './viewport';
import { itemEnd } from './visible-items';

/**
 * The overview strip under the timeline: the whole sequence at a glance, with the visible window
 * as a rectangle to drag (scroll) or resize by its edges (zoom).
 */

/** Seconds shown by the overview: the content and the visible window, plus a margin. */
export function overviewSeconds(viewport: Viewport, contentEnd: Flicks): number {
  const visibleEnd =
    flicksToSeconds(viewport.scrollTime) + viewport.width / viewport.pixelsPerSecond;
  return Math.max(flicksToSeconds(contentEnd) * 1.05, visibleEnd, 1);
}

/** The visible window as a span of the overview, in pixels. */
export function windowRect(
  viewport: Viewport,
  contentEnd: Flicks,
  overviewWidth: number,
): { readonly left: number; readonly width: number } {
  const scale = overviewWidth / overviewSeconds(viewport, contentEnd);
  return {
    left: flicksToSeconds(viewport.scrollTime) * scale,
    width: Math.max(4, (viewport.width / viewport.pixelsPerSecond) * scale),
  };
}

/** Time at `x` pixels of the overview. */
export function overviewTime(
  x: number,
  viewport: Viewport,
  contentEnd: Flicks,
  overviewWidth: number,
): Flicks {
  const seconds = (x / overviewWidth) * overviewSeconds(viewport, contentEnd);
  return clampFlicks(Math.max(0, seconds * 705_600_000));
}

/**
 * The viewport after dragging the window's `edge` (or the whole window, `move`) to new overview
 * pixels `left` and `right`: moving scrolls, resizing zooms so the window keeps its other edge.
 */
export function viewportFromWindow(
  viewport: Viewport,
  span: { readonly left: number; readonly right: number },
  contentEnd: Flicks,
  overviewWidth: number,
  limits: ZoomLimits,
): Viewport {
  const total = overviewSeconds(viewport, contentEnd);
  const startSeconds = Math.max(0, (span.left / overviewWidth) * total);
  const endSeconds = Math.max(startSeconds + 1e-6, (span.right / overviewWidth) * total);
  const pixelsPerSecond = clampZoom(viewport.width / (endSeconds - startSeconds), limits);
  return {
    ...viewport,
    pixelsPerSecond,
    scrollTime: clampFlicks(startSeconds * 705_600_000),
  };
}

/** One item drawn in the overview: a thin bar on its track's line. */
export interface OverviewBar {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly kind: TrackKind;
}

/** The bars of every item, one line per track (top to bottom), across `width` × `height`. */
export function overviewBars(
  rows: readonly TrackRow[],
  itemsByTrack: ReadonlyMap<string, readonly ItemEntity[]>,
  totalSeconds: number,
  width: number,
  height: number,
): OverviewBar[] {
  const xOf = (time: Flicks) => (flicksToSeconds(time) / totalSeconds) * width;
  const lineHeight = (height - 4) / Math.max(rows.length, 1);
  return rows.flatMap((row, index) =>
    (itemsByTrack.get(row.track.id) ?? []).map((item) => ({
      x: xOf(item.timelineRange.start),
      y: 2 + index * lineHeight,
      width: Math.max(1, xOf(itemEnd(item)) - xOf(item.timelineRange.start)),
      kind: row.track.kind,
    })),
  );
}

/** Position of the window along the overview, from 0 (start) to 100 (end), for assistive tech. */
export function windowPercent(
  rect: { readonly left: number; readonly width: number },
  overviewWidth: number,
): number {
  const room = overviewWidth - rect.width;
  return room <= 0 ? 0 : Math.round(Math.min(100, Math.max(0, (rect.left / room) * 100)));
}
