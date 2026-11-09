import type { ItemEntity, ItemId, SequenceEntity } from '@se/protocol';

import { previewedItem, usePropertyPreview } from '../../../shared/animation';
import { useCaptionPreview, type CaptionPreview } from '../../../shared/caption-preview';
import { useProjectSelector } from '../../../shared/project-store';
import { sceneOf, type PlaybackScene } from '../services/viewer-scene';

/**
 * The scene of `sequence` from the read-only project mirror, rebuilt when its entities change
 * (every patch replaces the records it touches), with the values of a continuous edit in progress
 * (gizmo or inspector drag, text being typed) drawn over the document's.
 */
export function useViewerScene(sequence: SequenceEntity): PlaybackScene | null {
  const tracks = useProjectSelector((state) => state.mirror?.tracks);
  const items = useProjectSelector((state) => state.mirror?.items);
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const captionStyles = useProjectSelector((state) => state.mirror?.project.captionStyles);
  const templates = useProjectSelector((state) => state.mirror?.project.templates);
  const preview = usePropertyPreview();
  const captionPreview = useCaptionPreview();
  if (tracks === undefined || items === undefined || assets === undefined) return null;
  const previewed = new Set([
    ...Object.keys(preview.animations),
    ...Object.keys(preview.contents),
    ...Object.keys(preview.templateParams),
  ]);
  let shown: Readonly<Record<ItemId, ItemEntity>> = items;
  if (previewed.size > 0) {
    const copy: Record<ItemId, ItemEntity> = { ...items };
    for (const id of previewed) {
      const item = items[id];
      if (item !== undefined) copy[id] = previewedItem(item, preview);
    }
    shown = copy;
  }
  if (captionPreview !== null) {
    return sceneOf({
      sequence,
      tracks,
      items: withPreviewStyle(shown, captionPreview),
      assets,
      captionStyles: [...(captionStyles ?? []), captionPreview.style],
      templates,
    });
  }
  return sceneOf({ sequence, tracks, items: shown, assets, captionStyles, templates });
}

/** `items` whose previewed captions point at the preview's style. */
function withPreviewStyle(
  items: Readonly<Record<ItemId, ItemEntity>>,
  preview: CaptionPreview,
): Readonly<Record<ItemId, ItemEntity>> {
  const copy: Record<ItemId, ItemEntity> = { ...items };
  for (const id of preview.itemIds) {
    const item = items[id];
    if (item?.kind.kind === 'caption') {
      copy[id] = { ...item, kind: { ...item.kind, styleId: preview.style.id } };
    }
  }
  return copy;
}
