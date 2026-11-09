import type { CoreClient, LibraryBrandKit } from '@se/protocol';
import { useEffect } from 'react';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { brandKitsApi } from '../api';

/** The library's brand kits as last listed (`null` until the first listing). */
export const brandKitsStore = createStore<{
  readonly kits: readonly LibraryBrandKit[] | null;
}>()(() => ({ kits: null }));

let loading: Promise<void> | null = null;

/** Lists the kits again (one listing at a time). */
export function reloadBrandKits(client: Pick<CoreClient, 'send'>): Promise<void> {
  loading ??= brandKitsApi(client)
    .list()
    .then((kits) => {
      brandKitsStore.setState({ kits });
    })
    .finally(() => {
      loading = null;
    });
  return loading;
}

/** Forgets the listing (tests). */
export function resetBrandKits(): void {
  brandKitsStore.setState({ kits: null });
  loading = null;
}

/**
 * The library's kits (`null` until listed), listed on first use and again whenever the core says
 * they changed (`libraryChanged` with `brandKits`).
 */
export function useBrandKits(): readonly LibraryBrandKit[] | null {
  const client = useCoreClient();
  const kits = useStore(brandKitsStore, (state) => state.kits);
  useEffect(() => {
    if (kits === null) reloadBrandKits(client).catch(() => undefined);
  }, [client, kits]);
  useCoreEvent('libraryChanged', (event) => {
    if (event.brandKits) reloadBrandKits(client).catch(() => undefined);
  });
  return kits;
}
