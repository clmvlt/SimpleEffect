import type { FontFamily } from '@se/protocol';
import { useEffect } from 'react';
import { useStore } from 'zustand';

import { useCoreClient } from '../core';
import { fontCatalogStore, loadFontCatalog } from './font-catalog-store';

const NONE: readonly FontFamily[] = [];

/** The font families of the catalogue (empty until the core answered), loading it on first use. */
export function useFontFamilies(): readonly FontFamily[] {
  const client = useCoreClient();
  const families = useStore(fontCatalogStore, (state) => state.families);
  useEffect(() => {
    if (families === null) loadFontCatalog(client).catch(() => undefined);
  }, [client, families]);
  return families ?? NONE;
}
