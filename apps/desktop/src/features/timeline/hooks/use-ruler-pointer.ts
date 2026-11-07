import type { Flicks, FrameRate } from '@se/protocol';
import { useRef, type PointerEvent } from 'react';
import { useStore } from 'zustand';

import { snapToFrame } from '../../../shared/time';
import { snapTime } from '../services/edit-points';
import { pixelsToDuration, xToTime, type Viewport } from '../services/viewport';
import { timelineViewStore } from '../store/timeline-view-store';
import type { InOutControls } from './use-in-out';

/** Pixels within which the dragged playhead snaps to a candidate. */
const SNAP_PIXELS = 8;

/** What a press on the ruler acts on. */
export interface RulerPointerTargets {
  readonly viewport: Viewport;
  readonly frameRate: FrameRate;
  /** Edit points, markers and in/out points the playhead may snap to. */
  readonly snapCandidates: () => Iterable<Flicks>;
  readonly movePlayhead: (time: Flicks) => void;
  readonly inOut: InOutControls;
}

/**
 * Pointer handling of the ruler: pressing or dragging moves the playhead (snapping to edit points,
 * markers and in/out points when snapping is on); with Maj, dragging draws the in/out span.
 */
export function useRulerPointer({
  viewport,
  frameRate,
  snapCandidates,
  movePlayhead,
  inOut,
}: RulerPointerTargets) {
  const snapping = useStore(timelineViewStore, (state) => state.snapping);
  const spanStart = useRef<Flicks | null>(null);
  const pressed = useRef(false);

  const timeAt = (event: PointerEvent<HTMLElement>) =>
    xToTime(event.clientX - event.currentTarget.getBoundingClientRect().left, viewport);

  const apply = (event: PointerEvent<HTMLElement>) => {
    const time = timeAt(event);
    if (spanStart.current !== null) {
      inOut.markSpan(spanStart.current, snapToFrame(time, frameRate, 'nearest'));
      return;
    }
    const threshold = pixelsToDuration(SNAP_PIXELS, viewport);
    movePlayhead(snapping ? snapTime(time, snapCandidates(), threshold) : time);
  };

  return {
    onPointerDown: (event: PointerEvent<HTMLElement>) => {
      if (event.button !== 0) return;
      event.currentTarget.setPointerCapture(event.pointerId);
      pressed.current = true;
      spanStart.current = event.shiftKey ? snapToFrame(timeAt(event), frameRate, 'nearest') : null;
      apply(event);
    },
    onPointerMove: (event: PointerEvent<HTMLElement>) => {
      if (pressed.current) apply(event);
    },
    onPointerUp: () => {
      pressed.current = false;
      spanStart.current = null;
    },
  };
}
