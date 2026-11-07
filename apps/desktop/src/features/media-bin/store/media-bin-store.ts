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
}

/** The media bin store. */
export const mediaBinStore = createStore<MediaBinState>()(() => ({
  view: 'grid',
  kind: 'all',
  query: '',
  selectedAssetId: null,
}));

/** Changes part of the media bin's UI state. */
export function updateMediaBin(changes: Partial<MediaBinState>): void {
  mediaBinStore.setState(changes);
}
