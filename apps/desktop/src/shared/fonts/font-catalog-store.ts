import {
  fontMediaPath,
  mediaUrl,
  type CoreClient,
  type CoreEndpoint,
  type FontFamily,
} from '@se/protocol';
import type { CatalogFace } from '@se/engine/fonts';
import { createStore } from 'zustand/vanilla';

/** The font catalogue of the running core (bundled and installed fonts). */
export interface FontCatalogState {
  /** `null` until `listFonts` answered. */
  readonly families: readonly FontFamily[] | null;
}

/** The catalogue store, shared by the inspector (font menu) and the viewer (font loading). */
export const fontCatalogStore = createStore<FontCatalogState>()(() => ({ families: null }));

let loading: Promise<void> | null = null;

/**
 * Asks the core for its fonts once (again after a failure). Listing installed fonts may take a
 * moment on the core's first call: callers do not wait for it.
 */
export function loadFontCatalog(client: Pick<CoreClient, 'send'>): Promise<void> {
  if (fontCatalogStore.getState().families !== null) return Promise.resolve();
  loading ??= client
    .send({ kind: 'listFonts' })
    .then((result) => {
      fontCatalogStore.setState({ families: result.families });
    })
    .finally(() => {
      loading = null;
    });
  return loading;
}

/** The catalogue's faces with the URLs serving them on `endpoint`, for the engine's font registry. */
export function catalogFaces(
  families: readonly FontFamily[],
  endpoint: CoreEndpoint,
): CatalogFace[] {
  return families.flatMap((family) =>
    family.faces.map((face) => ({
      family: family.name,
      url: mediaUrl(endpoint, fontMediaPath(face.id)),
      weightMin: face.weightMin,
      weightMax: face.weightMax,
      italic: face.italic,
      unicodeRange: face.unicodeRange,
      format: face.format,
    })),
  );
}

/** Forgets the catalogue (tests). */
export function resetFontCatalog(): void {
  fontCatalogStore.setState({ families: null });
}
