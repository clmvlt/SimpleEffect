import type { Asset } from '@se/protocol';
import { useEffect, useRef, type DragEvent } from 'react';
import { useStore } from 'zustand';

import { endAssetDrag, startAssetDrag } from '../../../shared/drag';
import { mediaBinStore } from '../store/media-bin-store';

/**
 * Props of a media bin entry (card or row) for drag and drop and reveals: the asset drags to
 * the timeline, and the entry scrolls into view when another panel reveals its asset.
 */
export function useAssetEntry(asset: Asset, selected: boolean) {
  const element = useRef<HTMLButtonElement>(null);
  const revealRequest = useStore(mediaBinStore, (state) => state.revealRequest);
  useEffect(() => {
    if (selected && revealRequest > 0) element.current?.scrollIntoView({ block: 'nearest' });
  }, [selected, revealRequest]);
  return {
    ref: element,
    draggable: true,
    onDragStart: (event: DragEvent<HTMLElement>) => {
      startAssetDrag(event.dataTransfer, [asset.id]);
    },
    onDragEnd: () => {
      endAssetDrag();
    },
  };
}
