/**
 * Caption looks previewed before they are applied (prompt 23): the captions feature writes them
 * (a hovered preset, a caption dragged in the viewer), the viewer draws them.
 */
export {
  PREVIEW_CAPTION_STYLE_ID,
  captionPreviewStore,
  clearCaptionPreview,
  previewCaptionStyle,
  useCaptionPreview,
} from './caption-preview-store';
export type { CaptionPreview } from './caption-preview-store';
