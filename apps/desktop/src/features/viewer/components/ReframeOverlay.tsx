import type { Asset, ItemId, SequenceEntity, Vec2 } from '@se/protocol';
import { clipSourceTime } from '@se/engine/evaluate';
import { Button } from '@se/ui';
import { useSyncExternalStore } from 'react';
import { useTranslation } from 'react-i18next';

import { useProjectSelector } from '../../../shared/project-store';
import { stopReframing } from '../../../shared/reframe';
import { usePlayheadTime } from '../../timeline';
import { useReframePointer } from '../hooks/use-reframe-pointer';
import { useThumbnailSheet } from '../hooks/use-thumbnail-sheet';
import {
  boxRectTransform,
  pictureCorners,
  pictureRect,
  type CroppedLayer,
} from '../services/reframe-view';
import { nearestTile, tileBackground } from '../services/sheet-tile';
import type { ViewerEngineController } from '../services/viewer-engine-controller';

/** The drawn layer of `itemId` when it crops its picture (a fill mode), else `null`. */
export function useCroppedLayer(
  controller: ViewerEngineController,
  itemId: ItemId,
): CroppedLayer | null {
  const layout = useSyncExternalStore(controller.subscribeLayout, controller.getLayout);
  const layer = layout?.layers.find((candidate) => candidate.itemId === itemId);
  if (layer?.crop == null) return null;
  return { itemId: layer.itemId, transform: layer.transform, box: layer.box, crop: layer.crop };
}

const points = (list: readonly Vec2[], scale: number) =>
  list.map((point) => `${String(point.x * scale)},${String(point.y * scale)}`).join(' ');

/** Props of the reframing components. */
export interface ReframeProps {
  readonly controller: ViewerEngineController;
  readonly sequence: SequenceEntity;
  readonly itemId: ItemId;
  /** CSS pixels per frame pixel. */
  readonly displayScale: number;
}

/**
 * Behind the picture while reframing: the clip's thumbnail nearest the playhead stretched over
 * its whole picture, dimmed, so the parts the frame crops away show around it.
 */
export function ReframeBackdrop({ controller, sequence, itemId, displayScale }: ReframeProps) {
  const layer = useCroppedLayer(controller, itemId);
  const item = useProjectSelector((state) => state.mirror?.items[itemId]);
  const asset = useProjectSelector((state) =>
    item?.kind.kind === 'mediaClip' ? state.mirror?.assets[item.kind.assetId] : undefined,
  );
  if (layer === null || item?.kind.kind !== 'mediaClip' || asset === undefined) return null;
  return (
    <BackdropTile
      asset={asset}
      layer={layer}
      sequence={sequence}
      clip={item.kind}
      range={item.timelineRange}
      displayScale={displayScale}
    />
  );
}

function BackdropTile({
  asset,
  layer,
  sequence,
  clip,
  range,
  displayScale,
}: {
  readonly asset: Asset;
  readonly layer: CroppedLayer;
  readonly sequence: SequenceEntity;
  readonly clip: Parameters<typeof clipSourceTime>[0];
  readonly range: Parameters<typeof clipSourceTime>[1];
  readonly displayScale: number;
}) {
  const sheet = useThumbnailSheet(asset);
  const playhead = usePlayheadTime(sequence.id);
  if (sheet === null) return null;
  const rect = pictureRect(layer);
  const index = nearestTile(sheet.sheet, clipSourceTime(clip, range, playhead));
  if (index < 0) return null;
  const background = tileBackground(sheet.sheet, index, rect.width, rect.height);
  return (
    <div
      aria-hidden
      data-reframe-backdrop=""
      className="pointer-events-none absolute top-0 left-0 bg-no-repeat opacity-40"
      style={{
        width: rect.width,
        height: rect.height,
        transformOrigin: '0 0',
        transform: boxRectTransform(layer, sequence.format, rect, displayScale),
        backgroundImage: `url("${sheet.imageUrl}")`,
        backgroundSize: background.size,
        backgroundPosition: background.position,
      }}
    />
  );
}

/**
 * Over the picture while reframing a clip that fills the frame: the outline of its whole
 * picture; dragging moves the picture under the frame to choose the part kept (the focus).
 * Enter, Escape or « Terminer » leave the mode.
 */
export function ReframeOverlay({ controller, sequence, itemId, displayScale }: ReframeProps) {
  const { t } = useTranslation();
  const layer = useCroppedLayer(controller, itemId);
  const pointer = useReframePointer(sequence, layer, displayScale);
  return (
    <div
      className="absolute inset-0 cursor-grab active:cursor-grabbing"
      data-viewer-reframe=""
      onPointerDown={pointer.onPointerDown}
      onPointerMove={pointer.onPointerMove}
      onPointerUp={pointer.onPointerUp}
      onDoubleClick={stopReframing}
    >
      {layer !== null && (
        <svg
          aria-hidden
          className="pointer-events-none absolute inset-0 size-full overflow-visible"
        >
          <polygon
            points={points(pictureCorners(layer, sequence.format), displayScale)}
            className="fill-none stroke-accent"
            strokeWidth={1.5}
            strokeDasharray="6 4"
          />
        </svg>
      )}
      <div className="absolute top-2 left-1/2 flex -translate-x-1/2 items-center gap-2 rounded-md bg-overlay/90 px-2 py-1 text-xs text-text shadow-overlay">
        <span>
          {layer === null ? t('editor.viewer.reframe.offClip') : t('editor.viewer.reframe.hint')}
        </span>
        <Button
          size="xs"
          variant="secondary"
          onPointerDown={(event) => {
            event.stopPropagation();
          }}
          onClick={stopReframing}
        >
          {t('editor.viewer.reframe.done')}
        </Button>
      </div>
    </div>
  );
}
