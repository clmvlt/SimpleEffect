import type { SequenceEntity } from '@se/protocol';
import type { RenderScene } from '@se/engine/preview';

import { useProjectSelector } from '../../../shared/project-store';
import { sceneOf } from '../services/viewer-scene';

/**
 * The scene of `sequence` from the read-only project mirror, rebuilt when its entities change
 * (every patch replaces the records it touches).
 */
export function useViewerScene(sequence: SequenceEntity): RenderScene | null {
  const tracks = useProjectSelector((state) => state.mirror?.tracks);
  const items = useProjectSelector((state) => state.mirror?.items);
  const assets = useProjectSelector((state) => state.mirror?.assets);
  if (tracks === undefined || items === undefined || assets === undefined) return null;
  return sceneOf({ sequence, tracks, items, assets });
}
