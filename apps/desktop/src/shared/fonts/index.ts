/**
 * The font catalogue of the core (`listFonts`): families for the inspector's font menu, faces
 * with their `/media/fonts` URLs for the engine's `FontRegistry`.
 */
export {
  catalogFaces,
  fontCatalogStore,
  loadFontCatalog,
  resetFontCatalog,
  type FontCatalogState,
} from './font-catalog-store';
export { useFontFamilies } from './use-font-families';
