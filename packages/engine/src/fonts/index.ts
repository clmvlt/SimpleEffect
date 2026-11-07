/**
 * `@se/engine/fonts`: the font catalogue's faces loaded with `FontFace` before texts are drawn
 * (preview and export alike), and the families a frame lacks.
 */
export {
  FontRegistry,
  createFontRegistry,
  type CatalogFace,
  type FontFaceFactory,
  type FontFaceLike,
  type FontFaceSetLike,
  type FontLoader,
} from './font-registry';
export type { FontRequest } from '../evaluate/frame-graph';
