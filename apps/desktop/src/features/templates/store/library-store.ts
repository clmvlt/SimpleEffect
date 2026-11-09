import type { CoreClient, LibraryTemplate } from '@se/protocol';
import { useEffect } from 'react';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { libraryApi } from '../api';

/** The template library as last listed (`null` until the first listing). */
export const libraryStore = createStore<{
  readonly entries: readonly LibraryTemplate[] | null;
}>()(() => ({ entries: null }));

let loading: Promise<void> | null = null;

/** Lists the library again (one listing at a time). */
export function reloadLibrary(client: Pick<CoreClient, 'send'>): Promise<void> {
  loading ??= libraryApi(client)
    .list()
    .then((entries) => {
      libraryStore.setState({ entries });
    })
    .finally(() => {
      loading = null;
    });
  return loading;
}

/** Forgets the listing (tests). */
export function resetLibrary(): void {
  libraryStore.setState({ entries: null });
  loading = null;
}

const NONE: readonly LibraryTemplate[] = [];

/**
 * The library's entries (empty until listed), listed on first use and again whenever the core
 * says the library changed (`libraryChanged`: a save, an import, a removal — in this window or
 * another).
 */
export function useLibraryTemplates(): readonly LibraryTemplate[] {
  const client = useCoreClient();
  const entries = useStore(libraryStore, (state) => state.entries);
  useEffect(() => {
    if (entries === null) reloadLibrary(client).catch(() => undefined);
  }, [client, entries]);
  useCoreEvent('libraryChanged', (event) => {
    if (event.templates) reloadLibrary(client).catch(() => undefined);
  });
  return entries ?? NONE;
}
