/**
 * `@se/engine/preview`: the viewer's engine (evaluate → decoder pool → PixiJS compositor) and the
 * frame renderer export will reuse. Loads PixiJS and Mediabunny: import it lazily from UI code.
 */
export {
  FrameRenderer,
  type FrameRequest,
  type RenderOutcome,
  type RenderScene,
} from './frame-renderer';
export type { MediaIssue, MediaIssueKind } from './media-issue';
export {
  createEngineWith,
  type PreviewEngine,
  type PreviewEngineOptions,
  type PreviewState,
} from './engine';
export { createPreviewEngine } from './preview-engine';
export type { CompositorBackground, RenderCanvas } from '../composite/compositor';
export type { DecoderPoolStats } from '../decode/decoder-pool';
export type { MediaLocator, MediaVariant } from '../decode/media-variant';
export { openPictureCounts } from '../decode/frame-tracker';
