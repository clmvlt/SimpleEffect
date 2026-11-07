import type { ItemAnimation, ItemEntity, SequenceEntity, Vec2 } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect, useRef, type PointerEvent as ReactPointerEvent } from 'react';
import { useTranslation } from 'react-i18next';

import {
  clearPropertyPreview,
  previewAnimations,
  propertyOf,
  withValueAt,
} from '../../../shared/animation';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { stopReframing } from '../../../shared/reframe';
import { frameDuration } from '../../../shared/time';
import { graphicsApi } from '../../graphics';
import { playheadStore, playheadTimeOf } from '../../timeline';
import { pannedFocus, type CroppedLayer } from '../services/reframe-view';

/** Pointer handlers of the reframing overlay. */
export interface ReframePointer {
  readonly onPointerDown: (event: ReactPointerEvent<HTMLDivElement>) => void;
  readonly onPointerMove: (event: ReactPointerEvent<HTMLDivElement>) => void;
  readonly onPointerUp: (event: ReactPointerEvent<HTMLDivElement>) => void;
}

/** The focus of `item` set to `focus` at the playhead (a keyframe when keyframed). */
function focusAnimation(
  item: ItemEntity,
  sequence: SequenceEntity,
  focus: Vec2,
): ItemAnimation | null {
  const current = propertyOf(item.kind, 'reframeFocus');
  if (current === null) return null;
  const playhead = playheadTimeOf(playheadStore.getState(), sequence.id);
  const local = (playhead - item.timelineRange.start) as typeof playhead;
  const tolerance = frameDuration(sequence.format.frameRate) / 2;
  const value = withValueAt(current, local, focus, tolerance);
  return { itemId: item.id, animation: { property: 'reframeFocus', value } };
}

/**
 * Dragging in the reframing overlay moves the picture under the frame: the focus follows
 * (`pannedFocus`), previewed while dragging (`previewAnimations`), sent as one
 * `setItemAnimations` on release (one undo step; a keyframe at the playhead when the focus is
 * keyframed). Escape cancels a drag, or leaves the mode when no drag runs; Enter leaves it.
 */
export function useReframePointer(
  sequence: SequenceEntity,
  layer: CroppedLayer | null,
  displayScale: number,
): ReframePointer {
  const { t } = useTranslation();
  const client = useCoreClient();
  const mirror = useProjectSelector((state) => state.mirror);
  const drag = useRef<{ from: Vec2; layer: CroppedLayer; change: ItemAnimation | null } | null>(
    null,
  );
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' && event.key !== 'Enter') return;
      if (drag.current !== null && event.key === 'Escape') {
        drag.current = null;
        clearPropertyPreview();
        return;
      }
      stopReframing();
    };
    window.addEventListener('keydown', onKey);
    return () => {
      window.removeEventListener('keydown', onKey);
    };
  }, []);
  const framePoint = (event: { clientX: number; clientY: number; currentTarget: Element }) => {
    const rect = event.currentTarget.getBoundingClientRect();
    return {
      x: (event.clientX - rect.left) / displayScale,
      y: (event.clientY - rect.top) / displayScale,
    };
  };
  return {
    onPointerDown: (event) => {
      if (event.button !== 0 || layer === null) return;
      event.currentTarget.setPointerCapture(event.pointerId);
      drag.current = { from: framePoint(event), layer, change: null };
    },
    onPointerMove: (event) => {
      const current = drag.current;
      const item = current === null ? undefined : mirror?.items[current.layer.itemId];
      if (current === null || item === undefined) return;
      const focus = pannedFocus(current.layer, sequence.format, current.from, framePoint(event));
      if (focus === null) return;
      const change = focusAnimation(item, sequence, focus);
      current.change = change;
      if (change !== null) previewAnimations([change]);
    },
    onPointerUp: () => {
      const current = drag.current;
      drag.current = null;
      if (current?.change == null || mirror === null) {
        clearPropertyPreview();
        return;
      }
      graphicsApi(client, mirror.projectId)
        .setItemAnimations([current.change])
        .then(clearPropertyPreview, (error: unknown) => {
          clearPropertyPreview();
          toast.error(t(errorMessageKey(error)));
        });
    },
  };
}
