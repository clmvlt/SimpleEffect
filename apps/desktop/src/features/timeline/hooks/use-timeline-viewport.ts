import type { Flicks, SequenceEntity } from '@se/protocol';
import { useEffect } from 'react';
import { useStore } from 'zustand';

import {
  clampScroll,
  DEFAULT_PIXELS_PER_SECOND,
  fitViewport,
  revealTime,
  scrollByPixels,
  timeToX,
  zoomAround,
  zoomLimits,
  type Viewport,
  type ZoomLimits,
} from '../services/viewport';
import { playheadStore, playheadTimeOf } from '../store/playhead-store';
import {
  sequenceViewOf,
  timelineViewStore,
  updateSequenceView,
} from '../store/timeline-view-store';

/** Ratio of one zoom step (buttons and `+`/`-`). */
export const ZOOM_STEP = 1.25;

/** The viewport of the active sequence and the ways to move it. */
export interface TimelineViewportApi {
  readonly viewport: Viewport;
  readonly limits: ZoomLimits;
  /** Zooms to `pixelsPerSecond` around `anchorX` (default: the playhead if visible, else the middle). */
  readonly zoomTo: (pixelsPerSecond: number, anchorX?: number) => void;
  /** Zooms by `factor` (> 1 = closer) around `anchorX` (same default). */
  readonly zoomBy: (factor: number, anchorX?: number) => void;
  /** Scrolls by `pixels` (positive = later). */
  readonly scrollBy: (pixels: number) => void;
  /** Replaces the viewport (minimap drags), keeping the scroll in range. */
  readonly setViewport: (viewport: Viewport) => void;
  /** Shows the whole sequence. */
  readonly fit: () => void;
  /** Scrolls so `time` is visible. */
  readonly reveal: (time: Flicks) => void;
}

/** The viewport of `sequence`, stored per sequence; fits the sequence the first time it shows. */
export function useTimelineViewport(
  sequence: SequenceEntity,
  contentEnd: Flicks,
): TimelineViewportApi {
  const view = useStore(timelineViewStore, (state) => sequenceViewOf(state, sequence.id));
  const width = useStore(timelineViewStore, (state) => state.lanesWidth);
  const limits = zoomLimits(sequence.format.frameRate);
  const viewport: Viewport = {
    pixelsPerSecond: view.pixelsPerSecond ?? DEFAULT_PIXELS_PER_SECOND,
    scrollTime: view.scrollTime,
    width,
  };
  const store = (next: Viewport) => {
    const clamped = clampScroll(next, contentEnd);
    updateSequenceView(sequence.id, {
      pixelsPerSecond: clamped.pixelsPerSecond,
      scrollTime: clamped.scrollTime,
    });
  };
  const current = (): Viewport => {
    const state = timelineViewStore.getState();
    const latest = sequenceViewOf(state, sequence.id);
    return {
      pixelsPerSecond: latest.pixelsPerSecond ?? DEFAULT_PIXELS_PER_SECOND,
      scrollTime: latest.scrollTime,
      width: state.lanesWidth,
    };
  };
  const defaultAnchor = (latest: Viewport) => {
    const x = timeToX(playheadTimeOf(playheadStore.getState(), sequence.id), latest);
    return x >= 0 && x <= latest.width ? x : latest.width / 2;
  };
  const zoomTo = (pixelsPerSecond: number, anchorX?: number) => {
    const latest = current();
    store(zoomAround(latest, pixelsPerSecond, anchorX ?? defaultAnchor(latest), limits));
  };
  const needsFit = view.pixelsPerSecond === null && width > 0;
  useEffect(() => {
    if (needsFit) store(fitViewport(current(), contentEnd, limits));
  });
  return {
    viewport,
    limits,
    zoomTo,
    zoomBy: (factor, anchorX) => {
      zoomTo(current().pixelsPerSecond * factor, anchorX);
    },
    scrollBy: (pixels) => {
      store(scrollByPixels(current(), pixels, contentEnd));
    },
    setViewport: store,
    fit: () => {
      store(fitViewport(current(), contentEnd, limits));
    },
    reveal: (time) => {
      store(revealTime(current(), time, contentEnd));
    },
  };
}
