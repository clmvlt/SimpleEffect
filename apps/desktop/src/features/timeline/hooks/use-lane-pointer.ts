import type { ItemEdge, ItemEntity, ItemId } from '@se/protocol';
import { useRef, useState, type PointerEvent } from 'react';

import { useShortcutRegistry } from '../../../shared/shortcuts';
import { gapAt } from '../services/gaps';
import { itemsInRect, marqueeSelection, type MarqueeRect } from '../services/marquee';
import {
  linkedItems,
  nextSelection,
  selectionModeOf,
  selectionOnPress,
  type SelectionMode,
} from '../services/selection';
import { rowAt } from '../services/track-layout';
import { trimGroup } from '../services/trim-placement';
import { xToTime } from '../services/viewport';
import {
  clearSelection,
  selectGap,
  selectionStore,
  selectItems,
  setActiveTrack,
} from '../store/selection-store';
import type { GestureContext, LaneGesture, LanePoint } from './gestures/gesture-context';
import { startGapGesture } from './gestures/gap-gesture';
import { startMoveGesture } from './gestures/move-gesture';
import { startTrimGesture } from './gestures/trim-gesture';

/** Pixels the pointer must travel before a press becomes a drag. */
const DRAG_THRESHOLD = 3;

/** What a press started, waiting for the pointer to travel or be released. */
type Press =
  | { readonly kind: 'marquee'; readonly base: readonly ItemId[]; readonly mode: SelectionMode }
  | {
      readonly kind: 'item';
      readonly item: ItemEntity;
      readonly before: readonly ItemId[];
      readonly mode: SelectionMode;
      readonly alone: boolean;
      readonly start: () => LaneGesture | null;
    }
  | { readonly kind: 'gesture'; readonly start: () => LaneGesture | null };

interface PointerState {
  readonly origin: LanePoint;
  readonly press: Press;
  dragging: boolean;
  gesture: LaneGesture | null;
}

/** Lane pixels of a pointer event (the lanes element scrolls with the tracks). */
function lanePoint(event: PointerEvent<HTMLElement>): LanePoint {
  const bounds = event.currentTarget.getBoundingClientRect();
  return { x: event.clientX - bounds.left, y: event.clientY - bounds.top };
}

/** Whether a `data-trim-edge` value names an edge. */
function isEdge(value: string | null): value is ItemEdge {
  return value === 'start' || value === 'end';
}

/** The item, trim edge or gap handle under a pointer event. */
function pressTarget(event: PointerEvent<HTMLElement>) {
  const target = event.target instanceof Element ? event.target : null;
  const itemId = target?.closest<HTMLElement>('[data-item-id]')?.dataset['itemId'] ?? null;
  const edge = target?.closest<HTMLElement>('[data-trim-edge]')?.dataset['trimEdge'] ?? null;
  const gapHandle = target !== null && target.closest('[data-gap-handle]') !== null;
  return { itemId, edge: isEdge(edge) ? edge : null, gapHandle };
}

/**
 * Everything the pointer does in the lanes: selecting items (Maj adds, Ctrl/⌘ toggles, Alt
 * ignores links) and gaps, the selection rectangle, and the drags (move, Alt copy, edge trims,
 * gap handle), each sent as one command on release. Escape cancels a drag.
 */
export function useLanePointer(context: () => GestureContext) {
  const { platform } = useShortcutRegistry();
  const apple = platform === 'apple';
  const [rect, setRect] = useState<MarqueeRect | null>(null);
  const pointer = useRef<PointerState | null>(null);
  const escape = useRef<((event: KeyboardEvent) => void) | null>(null);
  const modifiers = (event: PointerEvent<HTMLElement>) => ({
    insert: apple ? event.metaKey : event.ctrlKey,
    alt: event.altKey,
  });

  const itemPress = (item: ItemEntity, edge: ItemEdge | null, event: PointerEvent<HTMLElement>) => {
    const { model } = context();
    const ids = model.items.map((candidate) => candidate.id);
    const before = selectionStore.getState().itemIds;
    const mode = selectionModeOf(event, apple);
    selectItems(selectionOnPress(before, linkedItems(item, model.itemsById, ids, false), mode));
    setActiveTrack(item.trackId);
    const origin = lanePoint(event);
    if (edge !== null) {
      const items = trimGroup(item, edge, model.items, event.altKey);
      return {
        kind: 'gesture',
        start: () => startTrimGesture(context, items, edge, origin),
      } as const;
    }
    const start = () => {
      const selected = new Set(selectionStore.getState().itemIds);
      const dragged = context().model.items.filter((candidate) => selected.has(candidate.id));
      return startMoveGesture(context, item, dragged, origin);
    };
    return { kind: 'item', item, before, mode, alone: event.altKey, start } as const;
  };

  const onPointerDown = (event: PointerEvent<HTMLDivElement>) => {
    if (event.button !== 0) return;
    const origin = lanePoint(event);
    event.currentTarget.setPointerCapture(event.pointerId);
    const { itemId, edge, gapHandle } = pressTarget(event);
    const item = itemId === null ? undefined : context().model.itemsById[itemId];
    const gap = selectionStore.getState().gap;
    let press: Press;
    if (gapHandle && gap !== null) {
      press = { kind: 'gesture', start: () => startGapGesture(context, gap, origin) };
    } else if (item !== undefined) {
      press = itemPress(item, edge, event);
    } else {
      const row = rowAt(context().layout, origin.y);
      if (row !== undefined) setActiveTrack(row.track.id);
      const base = selectionStore.getState().itemIds;
      press = { kind: 'marquee', base, mode: selectionModeOf(event, apple) };
    }
    pointer.current = { origin, press, dragging: false, gesture: null };
  };

  const onPointerMove = (event: PointerEvent<HTMLDivElement>) => {
    const current = pointer.current;
    if (current === null) return;
    const point = lanePoint(event);
    const travelled = Math.hypot(point.x - current.origin.x, point.y - current.origin.y);
    if (!current.dragging && travelled < DRAG_THRESHOLD) return;
    if (!current.dragging) {
      current.dragging = true;
      if (current.press.kind !== 'marquee') {
        current.gesture = current.press.start();
        listenForEscape();
      }
    }
    if (current.press.kind === 'marquee') {
      const next = { x0: current.origin.x, y0: current.origin.y, x1: point.x, y1: point.y };
      setRect(next);
      const { model, layout, viewport } = context();
      const touched = itemsInRect(next, layout, viewport, model.items);
      selectItems(marqueeSelection(current.press.base, touched, current.press.mode));
      return;
    }
    current.gesture?.move(point, modifiers(event));
  };

  const click = (press: Press, point: LanePoint) => {
    if (press.kind === 'item') {
      const { model } = context();
      const ids = model.items.map((candidate) => candidate.id);
      const clicked = linkedItems(press.item, model.itemsById, ids, press.alone);
      selectItems(nextSelection(press.before, clicked, press.mode));
    } else if (press.kind === 'marquee' && press.mode === 'replace') {
      const { model, layout, viewport } = context();
      const row = rowAt(layout, point.y);
      const items = row === undefined ? [] : (model.itemsByTrack.get(row.track.id) ?? []);
      const gap = row === undefined ? null : gapAt(items, row.track.id, xToTime(point.x, viewport));
      if (gap !== null && !row?.track.locked) selectGap(gap);
      else clearSelection();
    }
  };

  const onPointerUp = (event: PointerEvent<HTMLDivElement>) => {
    const current = pointer.current;
    pointer.current = null;
    setRect(null);
    stopListening();
    if (current === null) return;
    if (!current.dragging) {
      click(current.press, current.origin);
      return;
    }
    void current.gesture?.finish(lanePoint(event), modifiers(event));
  };

  const onPointerCancel = () => {
    pointer.current?.gesture?.cancel();
    pointer.current = null;
    setRect(null);
    stopListening();
  };

  // Escape cancels a drag in progress: a key listener lives only during drags (cancelling a
  // gesture is not a command, so it is not a shortcut action and never steals Escape otherwise).
  function listenForEscape() {
    stopListening();
    const listener = (event: KeyboardEvent) => {
      if (event.key !== 'Escape') return;
      event.preventDefault();
      event.stopPropagation();
      onPointerCancel();
    };
    escape.current = listener;
    window.addEventListener('keydown', listener, true);
  }

  function stopListening() {
    if (escape.current !== null) window.removeEventListener('keydown', escape.current, true);
    escape.current = null;
  }

  return {
    rect,
    laneHandlers: { onPointerDown, onPointerMove, onPointerUp, onPointerCancel },
  };
}
