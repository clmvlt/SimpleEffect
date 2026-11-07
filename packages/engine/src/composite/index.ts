/**
 * Compositing: the {@link Compositor} interface, its PixiJS implementation and the pure geometry
 * both share.
 */
export type {
  Compositor,
  CompositorBackground,
  Picture,
  PictureSet,
  RenderCanvas,
} from './compositor';
export {
  layerPlacement,
  outputScale,
  pictureFit,
  type LayerPlacement,
  type PictureFit,
  type PictureRotation,
} from './layout';
export { createPixiCompositor, type PixiCompositorOptions } from './pixi/pixi-compositor';
