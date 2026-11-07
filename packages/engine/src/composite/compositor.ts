import type { ItemId } from '@se/protocol';

import type { FrameGraph, Size } from '../evaluate/frame-graph';
import type { PictureRotation } from './layout';

/** Decoded pixels of a picture layer, borrowed for the duration of one `render` call. */
export interface Picture {
  /** A `VideoFrame` (video) or an `ImageBitmap` (still image). */
  readonly source: VideoFrame | ImageBitmap;
  /** Clockwise display rotation of the pixels (phone footage). */
  readonly rotation: PictureRotation;
  /** Whether the pixels are mirrored horizontally after the rotation. */
  readonly mirrored: boolean;
}

/** Pictures by the id of the item whose layer shows them; a missing entry draws nothing. */
export type PictureSet = ReadonlyMap<ItemId, Picture>;

/**
 * The box of each drawn layer in sequence pixels, by item id: what its transform places (anchor,
 * scale, rotation). Texts are measured while drawing; other layers report their frame-graph box.
 */
export type LayerBoxes = ReadonlyMap<ItemId, Size>;

/** What fills the frame behind the layers: black (preview, export) or nothing (alpha). */
export type CompositorBackground = 'black' | 'transparent';

/** Canvas the engine draws on: in the page or in a Worker. */
export type RenderCanvas = HTMLCanvasElement | OffscreenCanvas;

/**
 * Draws frame graphs on a canvas (ADR-0002: PixiJS behind this interface so it can be replaced).
 *
 * Lifetime rule: `render` uploads every picture before returning and keeps no reference that it
 * could read later, so the decoder pool may close the frames as soon as `render` returns.
 */
export interface Compositor {
  /** Output size in pixels; the sequence frame is scaled to fill it. */
  resize(width: number, height: number): void;
  setBackground(background: CompositorBackground): void;
  /**
   * Draws `graph` synchronously, with `pictures` for its video and image layers, and returns the
   * box of every layer drawn (the viewer's transform handles use them).
   */
  render(graph: FrameGraph, pictures: PictureSet): LayerBoxes;
  /** Frees GPU resources; the compositor cannot be used afterwards. */
  destroy(): void;
}
