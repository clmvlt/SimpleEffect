import { createFontRegistry, type FontRegistry } from '@se/engine/fonts';
import type { CoreEndpoint, FontFamily } from '@se/protocol';
import { useEffect } from 'react';

import { useCoreEndpoint } from '../core';
import { catalogFaces } from './font-catalog-store';
import { useFontFamilies } from './use-font-families';

/** The page's registry for UI previews and the catalogue it was given. */
interface DocumentFonts {
  readonly registry: FontRegistry;
  families: readonly FontFamily[];
  endpoint: CoreEndpoint;
}

let documentFonts: DocumentFonts | null = null;

/**
 * The registry of catalogue faces in the page's font set, created once (`null` where the page
 * cannot load fonts, as in jsdom) and given the catalogue again when it or the core changed.
 */
function documentRegistry(
  families: readonly FontFamily[],
  endpoint: CoreEndpoint,
): FontRegistry | null {
  if (typeof FontFace === 'undefined' || typeof document === 'undefined') return null;
  if ((document as Partial<Document>).fonts === undefined) return null;
  if (documentFonts === null) {
    documentFonts = { registry: createFontRegistry(), families: [], endpoint };
  }
  if (documentFonts.families !== families || documentFonts.endpoint !== endpoint) {
    documentFonts.registry.setCatalog(catalogFaces(families, endpoint));
    documentFonts.families = families;
    documentFonts.endpoint = endpoint;
  }
  return documentFonts.registry;
}

/**
 * Loads catalogue families into the page so UI previews drawn with CSS (the brand kit's preview)
 * show them: bundled fonts are not installed on the system, and the viewer's registry only loads
 * what its frames use. Families the catalogue lacks are left to the system.
 */
export function useDocumentFonts(requested: readonly string[]): void {
  const families = useFontFamilies();
  const endpoint = useCoreEndpoint();
  const key = [...new Set(requested)].join('\n');
  useEffect(() => {
    if (endpoint === null || families.length === 0 || key === '') return;
    const registry = documentRegistry(families, endpoint);
    const requests = key.split('\n').map((family) => ({ family, weight: 400, italic: false }));
    void registry?.ensure(requests);
  }, [families, endpoint, key]);
}
