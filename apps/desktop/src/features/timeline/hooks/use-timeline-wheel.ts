import { useEffect, useEffectEvent, type RefObject } from 'react';

import { rowAt, type TrackLayout } from '../services/track-layout';
import { wheelResizedHeight } from '../services/track-height';
import { setTrackHeights } from '../store/timeline-view-store';
import type { TimelineViewportApi } from './use-timeline-viewport';

/** Zoom ratio per pixel of wheel delta (a notch of 100 px ≈ ×1.22). */
const ZOOM_PER_PIXEL = 0.002;

/** Elements of the timeline the wheel needs to locate the pointer. */
export interface WheelTargets {
  /** The timeline root receiving wheel events (holds the `[data-timeline-lanes]` element). */
  readonly root: RefObject<HTMLElement | null>;
  readonly layout: TrackLayout;
  readonly api: TimelineViewportApi;
}

/**
 * Wheel and trackpad gestures over the timeline: Ctrl/⌘ (or a pinch) zooms around the pointer,
 * Maj or a horizontal swipe scrolls in time, Alt resizes the track under the pointer, a plain
 * vertical wheel scrolls the tracks natively. Zoom and scroll are applied once per animation
 * frame, however many events arrive.
 */
export function useTimelineWheel({ root, layout, api }: WheelTargets): void {
  const onWheel = useEffectEvent((event: WheelEvent, pending: Pending) => {
    // The lanes give the x origin of the zoom anchor and the y origin of the track rows.
    const lanesBox = root.current?.querySelector('[data-timeline-lanes]')?.getBoundingClientRect();
    if (lanesBox === undefined) return;
    if (event.ctrlKey || event.metaKey) {
      event.preventDefault();
      pending.zoom *= Math.exp(-event.deltaY * ZOOM_PER_PIXEL);
      pending.anchorX = Math.max(0, event.clientX - lanesBox.left);
    } else if (event.altKey) {
      event.preventDefault();
      const row = rowAt(layout, event.clientY - lanesBox.top);
      if (row !== undefined)
        setTrackHeights([row.track.id], wheelResizedHeight(row.height, event.deltaY));
      return;
    } else if (event.shiftKey || Math.abs(event.deltaX) > Math.abs(event.deltaY)) {
      event.preventDefault();
      pending.scroll += event.shiftKey && event.deltaX === 0 ? event.deltaY : event.deltaX;
    } else {
      return;
    }
    pending.schedule();
  });
  const flush = useEffectEvent((pending: Pending) => {
    if (pending.zoom !== 1) api.zoomBy(pending.zoom, pending.anchorX);
    if (pending.scroll !== 0) api.scrollBy(pending.scroll);
    pending.zoom = 1;
    pending.scroll = 0;
  });
  useEffect(() => {
    const node = root.current;
    if (node === null) return undefined;
    let frame: number | null = null;
    const pending: Pending = {
      zoom: 1,
      scroll: 0,
      anchorX: 0,
      schedule: () => {
        frame ??= requestAnimationFrame(() => {
          frame = null;
          flush(pending);
        });
      },
    };
    const listener = (event: WheelEvent) => {
      onWheel(event, pending);
    };
    node.addEventListener('wheel', listener, { passive: false });
    return () => {
      node.removeEventListener('wheel', listener);
      if (frame !== null) cancelAnimationFrame(frame);
    };
  }, [root]);
}

/** Wheel deltas gathered until the next animation frame. */
interface Pending {
  zoom: number;
  scroll: number;
  anchorX: number;
  readonly schedule: () => void;
}
