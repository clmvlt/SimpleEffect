import type { CaptionStyle, ItemId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

/** Id of the style drawn during a preview (never a document id). */
export const PREVIEW_CAPTION_STYLE_ID = 'caption-style-preview';

/**
 * A caption look shown before it is applied: the viewer draws `itemIds` with `style` (a preset
 * hovered in the style panel, a caption being dragged) until the preview is cleared.
 */
export interface CaptionPreview {
  readonly itemIds: readonly ItemId[];
  readonly style: CaptionStyle;
}

/** The caption preview store (`null`: the document's styles). */
export const captionPreviewStore = createStore<{ readonly preview: CaptionPreview | null }>()(
  () => ({ preview: null }),
);

/** Shows `itemIds` drawn with `style` (its id is replaced by {@link PREVIEW_CAPTION_STYLE_ID}). */
export function previewCaptionStyle(itemIds: readonly ItemId[], style: CaptionStyle): void {
  captionPreviewStore.setState({
    preview: { itemIds, style: { ...style, id: PREVIEW_CAPTION_STYLE_ID } },
  });
}

/** Back to the document's styles. */
export function clearCaptionPreview(): void {
  if (captionPreviewStore.getState().preview !== null) {
    captionPreviewStore.setState({ preview: null });
  }
}

/** The caption preview shown. */
export function useCaptionPreview(): CaptionPreview | null {
  return useStore(captionPreviewStore, (state) => state.preview);
}
