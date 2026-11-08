import type { Flicks } from '@se/protocol';

import { createPixiCompositor } from '../composite/pixi/pixi-compositor';
import type { Compositor } from '../composite/compositor';
import { DecoderPool, type DecoderPoolOptions } from '../decode/decoder-pool';
import type { MediaLocator } from '../decode/media-variant';
import type { FontLoader } from '../fonts/font-registry';
import { FrameRenderer, type RenderScene } from './frame-renderer';
import type { MediaIssue } from './media-issue';

/** JPEG quality of stills (0–1): small enough for agents, sharp enough to read texts. */
export const STILL_JPEG_QUALITY = 0.85;

/** Options of {@link createStillRenderer}. */
export interface StillRendererOptions {
  /** Picture size in pixels (the sequence frame is scaled to fill it). */
  readonly width: number;
  readonly height: number;
  readonly locator: MediaLocator;
  /** Fonts to load before drawing texts. */
  readonly fonts?: FontLoader | undefined;
  readonly pool?: DecoderPoolOptions | undefined;
}

/** One still: the JPEG and what could not be drawn as asked. */
export interface Still {
  readonly jpeg: Blob;
  readonly issues: readonly MediaIssue[];
  readonly missingFonts: readonly string[];
}

/**
 * Draws stills of scenes offscreen and encodes them as JPEG: frames of the sequence for AI agents
 * (`get_frames`, ADR-0028), later thumbnails of exports. The same `FrameRenderer` and compositor as
 * the viewer (golden rule 7), on its own canvas and decoder pool, so it never disturbs playback.
 */
export interface StillRenderer {
  /** Draws `scene` at `time` (proxies when `proxy`) and encodes it. One still at a time. */
  render(scene: RenderScene, time: Flicks, proxy: boolean): Promise<Still>;
  /** Frees the canvas, the GPU context and every decoded picture. */
  destroy(): void;
}

/** Creates a still renderer; rejects when no GPU context is available. */
export async function createStillRenderer(options: StillRendererOptions): Promise<StillRenderer> {
  const canvas = new OffscreenCanvas(options.width, options.height);
  const compositor = await createPixiCompositor({
    canvas,
    width: options.width,
    height: options.height,
    background: 'black',
    // The picture is read back after compositing.
    preserveDrawingBuffer: true,
  });
  return createStillRendererWith(canvas, compositor, new DecoderPool(options.pool), options);
}

/** A still renderer over given parts (tests use a recording compositor and fake decoding). */
export function createStillRendererWith(
  canvas: OffscreenCanvas,
  compositor: Compositor,
  pool: DecoderPool,
  options: Pick<StillRendererOptions, 'locator' | 'fonts'>,
): StillRenderer {
  const renderer = new FrameRenderer(compositor, pool, options.locator);
  renderer.setFonts(options.fonts ?? null);
  let busy: Promise<unknown> = Promise.resolve();
  const drawOne = async (scene: RenderScene, time: Flicks, proxy: boolean): Promise<Still> => {
    const outcome = await renderer.render({ scene, time, proxy });
    if (outcome.kind !== 'drawn') throw new Error('the still was replaced before it was drawn');
    const jpeg = await canvas.convertToBlob({ type: 'image/jpeg', quality: STILL_JPEG_QUALITY });
    return { jpeg, issues: outcome.issues, missingFonts: outcome.missingFonts };
  };
  return {
    render(scene, time, proxy) {
      // One at a time: a second render would supersede the first in the frame renderer.
      const next = busy.then(() => drawOne(scene, time, proxy));
      busy = next.catch(() => undefined);
      return next;
    },
    destroy() {
      renderer.cancel();
      pool.destroy();
      compositor.destroy();
    },
  };
}
