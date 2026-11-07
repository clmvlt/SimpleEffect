import type { Asset } from '@se/protocol';
import { useStore } from 'zustand';

import { useProjectSelector } from '../../../shared/project-store';
import { filterAssets, sortAssets } from '../services/asset-list';
import { mediaBinStore } from '../store/media-bin-store';

const NO_ASSETS: Readonly<Record<string, Asset>> = {};

/** Every asset of the open project (import order) and those the filter and search keep. */
export function useMediaAssets() {
  const records = useProjectSelector((state) => state.mirror?.assets ?? NO_ASSETS);
  const kind = useStore(mediaBinStore, (state) => state.kind);
  const query = useStore(mediaBinStore, (state) => state.query);
  const all = sortAssets(records);
  return { all, visible: filterAssets(all, kind, query) };
}
