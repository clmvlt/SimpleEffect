import type { ItemEntity } from '@se/protocol';
import { useRef, useState, type PointerEvent } from 'react';

import { useShortcutRegistry } from '../../../shared/shortcuts';
import { itemsInRect, marqueeSelection, type MarqueeRect } from '../services/marquee';
import {
  linkedItems,
  nextSelection,
  selectionModeOf,
  type SelectionMode,
} from '../services/selection';
import type { TrackLayout } from '../services/track-layout';
import type { Viewport } from '../services/viewport';
import { clearSelection, selectionStore, selectItems } from '../store/selection-store';
import type { TimelineModel } from './use-timeline-model';

/** Pixels the pointer must travel before a press on empty lanes becomes a rectangle. */
const DRAG_THRESHOLD = 3;

interface Drag {
  readonly x0: number;
  readonly y0: number;
  readonly base: readonly string[];
  readonly mode: SelectionMode;
  moved: boolean;
}

/** Lane pixels of a pointer event (the lanes element scrolls with the tracks). */
function lanePoint(event: PointerEvent<HTMLElement>) {
  const bounds = event.currentTarget.getBoundingClientRect();
  return { x: event.clientX - bounds.left, y: event.clientY - bounds.top };
}

/**
 * Selection in the lanes: clicking an item (Maj adds, Ctrl/⌘ toggles, Alt ignores links),
 * drawing a rectangle from empty space, clicking empty space to deselect.
 */
export function useLaneSelection(model: TimelineModel, layout: TrackLayout, viewport: Viewport) {
  const { platform } = useShortcutRegistry();
  const [rect, setRect] = useState<MarqueeRect | null>(null);
  const drag = useRef<Drag | null>(null);
  const apple = platform === 'apple';
  const sequenceItemIds = model.items.map((item) => item.id);

  const onItemPointerDown = (item: ItemEntity, event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    event.stopPropagation();
    const clicked = linkedItems(item, model.itemsById, sequenceItemIds, event.altKey);
    const current = selectionStore.getState().itemIds;
    selectItems(nextSelection(current, clicked, selectionModeOf(event, apple)));
  };

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    const { x, y } = lanePoint(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    drag.current = {
      x0: x,
      y0: y,
      base: selectionStore.getState().itemIds,
      mode: selectionModeOf(event, apple),
      moved: false,
    };
  };

  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const current = drag.current;
    if (current === null) return;
    const { x, y } = lanePoint(event);
    if (!current.moved && Math.hypot(x - current.x0, y - current.y0) < DRAG_THRESHOLD) return;
    current.moved = true;
    const next = { x0: current.x0, y0: current.y0, x1: x, y1: y };
    setRect(next);
    const touched = itemsInRect(next, layout, viewport, model.items);
    selectItems(marqueeSelection(current.base, touched, current.mode));
  };

  const onPointerUp = () => {
    const current = drag.current;
    if (current !== null && !current.moved && current.mode === 'replace') clearSelection();
    drag.current = null;
    setRect(null);
  };

  return {
    rect,
    onItemPointerDown,
    laneHandlers: { onPointerDown, onPointerMove, onPointerUp, onPointerCancel: onPointerUp },
  };
}
