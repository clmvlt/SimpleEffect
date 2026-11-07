import type { ItemEntity, ItemId, SequenceEntity } from '@se/protocol';

import { previewedItem, usePropertyPreview } from '../../../shared/animation';
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
  const preview = usePropertyPreview();
  if (tracks === undefined || items === undefined || assets === undefined) return null;
  const previewed = new Set([...Object.keys(preview.animations), ...Object.keys(preview.contents)]);
  let shown: Readonly<Record<ItemId, ItemEntity>> = items;
  if (previewed.size > 0) {
    const copy: Record<ItemId, ItemEntity> = { ...items };
    for (const id of previewed) {
      const item = items[id];
      if (item !== undefined) copy[id] = previewedItem(item, preview);
    }
    shown = copy;
  }
  return sceneOf({ sequence, tracks, items: shown, assets });
}
