import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import type { KindFilter } from '../services/asset-list';

/** UI state of the media bin. */
export interface MediaBinState {
  readonly view: 'grid' | 'list';
  readonly kind: KindFilter;
  /** Text typed in the search field. */
  readonly query: string;
  /** Asset whose details are shown, or `null`. */
  readonly selectedAssetId: string | null;
  /** Bumped when another panel asks to show an asset: its card scrolls into view. */
  readonly revealRequest: number;
}

/** The media bin store. */
export const mediaBinStore = createStore<MediaBinState>()(() => ({
  view: 'grid',
  kind: 'all',
  query: '',
  selectedAssetId: null,
  revealRequest: 0,
}));

/** Changes part of the media bin's UI state. */
export function updateMediaBin(changes: Partial<MediaBinState>): void {
  mediaBinStore.setState(changes);
}

/**
 * Shows `assetId` in the media bin (« Afficher dans le panneau Médias »): selects it, clears the
 * filters that could hide it and scrolls its card into view.
 */
export function revealAsset(assetId: string): void {
  const { revealRequest } = mediaBinStore.getState();
  mediaBinStore.setState({
    selectedAssetId: assetId,
    kind: 'all',
    query: '',
    revealRequest: revealRequest + 1,
  });
}

/** The asset whose details the media bin shows (its selection), or `null`. */
export function useSelectedAssetId(): string | null {
  return useStore(mediaBinStore, (state) => state.selectedAssetId);
}
