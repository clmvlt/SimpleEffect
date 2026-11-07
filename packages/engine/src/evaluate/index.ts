/** Pure evaluation of a sequence at an instant into a {@link FrameGraph}. */
export {
  evaluate,
  type EvaluableItem,
  type EvaluableSequence,
  type EvaluableTrack,
} from './evaluate';
export {
  DISSOLVE_COMPOSITING,
  isPictureLayer,
  type Crop,
  type DissolveRole,
  type FrameGraph,
  type ImageLayer,
  type Layer,
  type PictureLayer,
  type ResolvedTransform,
  type ShapeLayer,
  type Size,
  type TextLayer,
  type VideoLayer,
} from './frame-graph';
export {
  DESIGN_REFERENCE_SIDE,
  assetDisplaySize,
  designScale,
  reframedBox,
  shapeBox,
} from './geometry';
export type { AssetTable } from './item-layer';
export {
  CENTRE_FOCUS,
  fillCrop,
  reframeScale,
  reframedPicture,
  type ReframeMode,
  type ReframedPicture,
} from './reframe';
export { clipSourceTime, fadeFactor } from './timing';
export { isInvisible, resolveTransform } from './transform';
export {
  playExtensions,
  trackTransitionSpans,
  transitionExtent,
  transitionProgress,
  transitionSpan,
  type EvaluableTransition,
  type PlayExtension,
  type TransitionSpan,
} from './transitions';
