import { createPixiCompositor } from '../composite/pixi/pixi-compositor';
import type { Compositor } from '../composite/compositor';
import { DecoderPool, type DecoderPoolOptions } from '../decode/decoder-pool';
import type { MediaLocator } from '../decode/media-variant';
import { createFontRegistry } from '../fonts/font-registry';
import { FrameRenderer } from '../preview/frame-renderer';
import { catalogFontSource } from '../templates/font-faces';
import { TemplateRuntime } from '../templates/template-runtime';
import { ExportFrameDrawer, rgbaReader, thumbnailOf } from './export-frames';
import type { ExportWorkerJob } from './export-messages';

/** The drawing side of an export: canvas, compositor, decoders, fonts and templates. */
export interface ExportRenderer {
  /** The canvas frames are drawn on (`null` for sound-only exports). */
  readonly canvas: OffscreenCanvas | null;
  readonly drawer: ExportFrameDrawer | null;
  readRgba(): Uint8Array;
  thumbnail(): Promise<Blob | null>;
  /** Frees the GPU context, decoders and template Workers. */
  destroy(): void;
}

/** The picture of an image asset for template Workers, or `null` when it cannot be read. */
async function loadImage(url: string | null): Promise<ImageBitmap | null> {
  if (url === null) return null;
  const response = await fetch(url);
  if (!response.ok) return null;
  return createImageBitmap(await response.blob());
}

/**
 * Builds the renderer of an export: an `OffscreenCanvas` at the output size with the preview's
 * PixiJS compositor (kept drawing buffer: frames are read back), its own decoder pool, the font
 * catalogue and — when the scene has motion templates — a template runtime of its own (the same
 * code as the preview's, golden rule 7). Works in a Worker and on a page; tests choose the
 * decoders (`poolOptions`).
 */
export async function createExportRenderer(
  job: Pick<ExportWorkerJob, 'settings' | 'scene' | 'fonts'>,
  locator: MediaLocator,
  poolOptions?: DecoderPoolOptions,
): Promise<ExportRenderer> {
  const video = job.settings.video;
  if (video === null) {
    return {
      canvas: null,
      drawer: null,
      readRgba: () => new Uint8Array(0),
      thumbnail: () => Promise.resolve(null),
      destroy: () => undefined,
    };
  }
  const canvas = new OffscreenCanvas(video.width, video.height);
  const compositor: Compositor = await createPixiCompositor({
    canvas,
    width: video.width,
    height: video.height,
    background: 'black',
    preserveDrawingBuffer: true,
  });
  const pool = new DecoderPool(poolOptions);
  const renderer = new FrameRenderer(compositor, pool, locator);
  const fonts = createFontRegistry();
  fonts.setCatalog(job.fonts);
  renderer.setFonts(fonts);
  const templates =
    (job.scene.sequence.templates?.length ?? 0) > 0
      ? new TemplateRuntime({
          fonts: catalogFontSource(job.fonts),
          images: (assetId) => loadImage(locator(assetId, 'original')),
        })
      : null;
  renderer.setTemplates(templates);
  renderer.setOutputSize(video.width, video.height);
  const drawer = new ExportFrameDrawer(renderer, pool, job.scene, locator);
  let readRgba: (() => Uint8Array) | null = null;
  return {
    canvas,
    drawer,
    readRgba: () => {
      readRgba ??= rgbaReader(canvas);
      return readRgba();
    },
    thumbnail: () => thumbnailOf(canvas),
    destroy: () => {
      drawer.stop();
      renderer.cancel();
      templates?.destroy();
      pool.destroy();
      compositor.destroy();
    },
  };
}
