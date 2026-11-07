import { createFontRegistry, type FontRegistry } from '@se/engine/fonts';
import { useEffect, useState } from 'react';

import { useCoreEndpoint } from '../../../shared/core';
import { catalogFaces, useFontFamilies } from '../../../shared/fonts';
import type { ViewerEngineController } from '../services/viewer-engine-controller';

/** A registry over the document's font set, or `null` where fonts cannot be loaded (jsdom). */
function makeRegistry(): FontRegistry | null {
  if (typeof FontFace === 'undefined' || typeof document === 'undefined') return null;
  return (document as Partial<Document>).fonts === undefined ? null : createFontRegistry();
}

/**
 * Gives the viewer's engine the font catalogue: its faces are registered with their
 * `/media/fonts` URLs (again when the core restarts) and loaded before texts are drawn.
 */
export function useViewerFonts(controller: ViewerEngineController): void {
  const families = useFontFamilies();
  const endpoint = useCoreEndpoint();
  const [registry] = useState(makeRegistry);
  useEffect(() => {
    if (registry === null || endpoint === null || families.length === 0) return;
    registry.setCatalog(catalogFaces(families, endpoint));
    controller.setFonts(registry);
  }, [controller, registry, families, endpoint]);
}
