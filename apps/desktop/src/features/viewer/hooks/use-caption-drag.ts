import type { CaptionStyle, ItemId, SequenceEntity, Vec2 } from '@se/protocol';
import { toast } from '@se/ui';
import { useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { clearCaptionPreview, previewCaptionStyle } from '../../../shared/caption-preview';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { captionsApi, captionsStore } from '../../captions';
import { draggedCaptionPosition } from '../services/caption-drag';
import type { SnapGuides } from '../services/gizmo-geometry';

/** A caption drag of the viewer: its style's position follows the pointer. */
export interface CaptionDrag {
  /** Starts dragging `itemId` from `point` (frame pixels); `false` when it is not a caption. */
  start(itemId: ItemId, selected: readonly ItemId[], point: Vec2): boolean;
  /** Whether a caption is being dragged. */
  isActive(): boolean;
  /** Previews the position for the pointer at `point`; returns the guides to draw. */
  move(point: Vec2, threshold: number): SnapGuides;
  /** Commits the position at `point` (one `updateCaptionStyle`), or cancels with `null`. */
  end(point: Vec2 | null, threshold: number): void;
}

interface Drag {
  readonly from: Vec2;
  readonly style: CaptionStyle;
  readonly targets: readonly ItemId[];
}

/**
 * Dragging a caption in `sequence`'s viewer moves the position of its style (bottom centre of the
 * block), for the captions the style panel's « Appliquer à » names: the whole track, or the
 * dragged caption and the selected ones (a shared style is copied for them by the core).
 */
export function useCaptionDrag(sequence: SequenceEntity): CaptionDrag {
  const { t } = useTranslation();
  const client = useCoreClient();
  const mirror = useProjectSelector((state) => state.mirror);
  const drag = useRef<Drag | null>(null);
  const frame = sequence.format;
  const positionAt = (point: Vec2, threshold: number) => {
    const current = drag.current;
    if (current === null) return null;
    return draggedCaptionPosition(current.style.position, current.from, point, frame, threshold);
  };
  return {
    start: (itemId, selected, point) => {
      const item = mirror?.items[itemId];
      if (mirror === null || item?.kind.kind !== 'caption') return false;
      const styleId = item.kind.styleId;
      const style = mirror.project.captionStyles.find((candidate) => candidate.id === styleId);
      if (style === undefined) return false;
      const track = mirror.tracks[item.trackId];
      const isCaption = (id: ItemId) => mirror.items[id]?.kind.kind === 'caption';
      const targets =
        captionsStore.getState().scope === 'track'
          ? (track?.itemIds ?? []).filter(isCaption)
          : [itemId, ...selected.filter((id) => id !== itemId && isCaption(id))];
      drag.current = { from: point, style, targets };
      return true;
    },
    isActive: () => drag.current !== null,
    move: (point, threshold) => {
      const result = positionAt(point, threshold);
      const current = drag.current;
      if (result === null || current === null) return { x: [], y: [] };
      previewCaptionStyle(current.targets, { ...current.style, position: result.position });
      return result.guides;
    },
    end: (point, threshold) => {
      const current = drag.current;
      const result = point === null ? null : positionAt(point, threshold);
      drag.current = null;
      if (current === null || result === null || mirror === null) {
        clearCaptionPreview();
        return;
      }
      captionsApi(client, mirror.projectId)
        .updateStyle(current.targets, { position: result.position })
        .then(clearCaptionPreview, (error: unknown) => {
          clearCaptionPreview();
          toast.error(t(errorMessageKey(error)));
        });
    },
  };
}
