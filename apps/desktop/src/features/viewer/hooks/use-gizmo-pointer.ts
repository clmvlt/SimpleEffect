import type { ItemEntity, ItemId, SequenceEntity, Vec2 } from '@se/protocol';
import { toast } from '@se/ui';
import {
  useRef,
  useState,
  type MouseEvent as ReactMouseEvent,
  type PointerEvent as ReactPointerEvent,
} from 'react';
import { useTranslation } from 'react-i18next';

import { clearPropertyPreview, previewAnimations } from '../../../shared/animation';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { frameDuration } from '../../../shared/time';
import { graphicsApi } from '../../graphics';
import { clearSelection, playheadStore, playheadTimeOf, selectItems } from '../../timeline';
import { hitTest, type PlacedLayer, type SnapGuides } from '../services/gizmo-geometry';
import {
  gestureStep,
  stepAnimations,
  type GizmoAction,
  type GizmoGesture,
} from '../services/gizmo-gesture';

/** Snap distance in CSS pixels. */
const SNAP_PIXELS = 8;
/** Pointer travel (CSS pixels) under which a press is a click, not a drag. */
const CLICK_PIXELS = 2;

/** What the gizmo's pointer handler needs. */
export interface GizmoPointerInput {
  readonly sequence: SequenceEntity;
  readonly layers: readonly PlacedLayer[];
  readonly selected: readonly ItemId[];
  /** CSS pixels per frame pixel. */
  readonly displayScale: number;
  readonly disabled: boolean;
}

/** The handlers of the gizmo overlay and the snap guides to draw. */
export interface GizmoPointer {
  readonly guides: SnapGuides;
  readonly onPointerDown: (event: ReactPointerEvent<HTMLDivElement>) => void;
  readonly onPointerMove: (event: ReactPointerEvent<HTMLDivElement>) => void;
  readonly onPointerUp: (event: ReactPointerEvent<HTMLDivElement>) => void;
  /** The text item under a double click, if any. */
  readonly textAt: (event: ReactMouseEvent<HTMLDivElement>) => ItemId | null;
}

function actionOf(handle: string | null): GizmoAction | null {
  if (handle === null) return null;
  if (handle === 'rotate') return { kind: 'rotate' };
  if (handle === 'anchor') return { kind: 'anchor' };
  const side = handle.replace('scale-', '');
  if (['n', 's', 'e', 'w', 'ne', 'nw', 'se', 'sw'].includes(side)) {
    return { kind: 'scale', handle: side as 'n' };
  }
  return null;
}

/**
 * The transform gizmo's pointer logic: a press on a handle scales, rotates or re-anchors the
 * selected item; a press on a layer selects it (Shift adds) and drags the selection; a press
 * elsewhere clears the selection. Drags only preview (`previewAnimations`); the release sends one
 * `setItemAnimations` (one undo step). Escape cancels; Ctrl/⌘ turns snapping off.
 */
export function useGizmoPointer(input: GizmoPointerInput): GizmoPointer {
  const { t } = useTranslation();
  const client = useCoreClient();
  const mirror = useProjectSelector((state) => state.mirror);
  const gesture = useRef<{ gesture: GizmoGesture; start: Vec2; moved: boolean } | null>(null);
  const stopEscape = useRef<(() => void) | null>(null);
  const [guides, setGuides] = useState<SnapGuides>({ x: [], y: [] });
  const { sequence, layers, displayScale } = input;
  const frame = sequence.format;
  const framePoint = (event: { clientX: number; clientY: number; currentTarget: Element }) => {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: (event.clientX - rect.left) / displayScale,
      y: (event.clientY - rect.top) / displayScale,
    };
  };
  const itemsOf = (ids: readonly ItemId[]) =>
    ids.map((id) => mirror?.items[id]).filter((item): item is ItemEntity => item !== undefined);
  const animationsAt = (event: ReactPointerEvent<HTMLDivElement>) => {
    const current = gesture.current;
    if (current === null) return null;
    const snapping = !(event.ctrlKey || event.metaKey);
    const step = gestureStep(current.gesture, frame, framePoint(event), {
      shift: event.shiftKey,
      snapThreshold: snapping ? SNAP_PIXELS / displayScale : 0,
    });
    const playhead = playheadTimeOf(playheadStore.getState(), sequence.id);
    const items = itemsOf(current.gesture.layers.map((layer) => layer.itemId));
    const tolerance = frameDuration(frame.frameRate) / 2;
    return { step, changes: stepAnimations(step, items, playhead, tolerance) };
  };
  const end = () => {
    gesture.current = null;
    stopEscape.current?.();
    stopEscape.current = null;
    setGuides({ x: [], y: [] });
  };
  /** Escape cancels the drag: no element has the focus during it, so the window listens. */
  const listenForEscape = () => {
    const onKey = (key: KeyboardEvent) => {
      if (key.key !== 'Escape') return;
      end();
      clearPropertyPreview();
    };
    window.addEventListener('keydown', onKey);
    stopEscape.current = () => {
      window.removeEventListener('keydown', onKey);
    };
  };
  return {
    guides,
    onPointerDown: (event) => {
      if (input.disabled || event.button !== 0) return;
      const point = framePoint(event);
      const target = event.target instanceof Element ? event.target : null;
      const handle =
        target?.closest('[data-gizmo-handle]')?.getAttribute('data-gizmo-handle') ?? null;
      let action = actionOf(handle);
      let ids: readonly ItemId[] = input.selected;
      if (action === null || ids.length !== 1) {
        action = { kind: 'move' };
        const hit = hitTest(layers, frame, point);
        if (hit === null) {
          clearSelection();
          return;
        }
        if (event.shiftKey) ids = ids.includes(hit) ? ids : [...ids, hit];
        else if (!ids.includes(hit)) ids = [hit];
        if (ids !== input.selected) selectItems(ids);
        // The item under the pointer leads (snapping).
        ids = [hit, ...ids.filter((id) => id !== hit)];
      }
      const dragged = ids
        .map((id) => layers.find((layer) => layer.itemId === id))
        .filter((layer): layer is PlacedLayer => layer !== undefined);
      gesture.current = {
        gesture: { action, from: point, layers: dragged },
        start: point,
        moved: false,
      };
      listenForEscape();
      event.currentTarget.setPointerCapture(event.pointerId);
    },
    onPointerMove: (event) => {
      const current = gesture.current;
      if (current === null) return;
      const point = framePoint(event);
      const travel =
        Math.hypot(point.x - current.start.x, point.y - current.start.y) * displayScale;
      if (!current.moved && travel < CLICK_PIXELS) return;
      current.moved = true;
      const result = animationsAt(event);
      if (result === null) return;
      setGuides(result.step.guides);
      previewAnimations(result.changes);
    },
    onPointerUp: (event) => {
      const current = gesture.current;
      if (current === null) return;
      const result = current.moved ? animationsAt(event) : null;
      end();
      if (result === null || result.changes.length === 0 || mirror === null) {
        clearPropertyPreview();
        return;
      }
      graphicsApi(client, mirror.projectId)
        .setItemAnimations(result.changes)
        .then(clearPropertyPreview, (error: unknown) => {
          clearPropertyPreview();
          toast.error(t(errorMessageKey(error)));
        });
    },
    textAt: (event) => {
      const hit = hitTest(layers, frame, framePoint(event));
      return hit !== null && mirror?.items[hit]?.kind.kind === 'text' ? hit : null;
    },
  };
}
