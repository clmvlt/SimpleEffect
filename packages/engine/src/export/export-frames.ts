import type { Flicks } from '@se/protocol';

import type { DecoderPool } from '../decode/decoder-pool';
import type { MediaLocator } from '../decode/media-variant';
import type { FrameRenderer, RenderScene } from '../preview/frame-renderer';
import { prefetchRequests, prefetchSpans } from '../preview/prefetch-plan';

/** Seconds of sequence the decoders read ahead of the frame being exported. */
const READ_AHEAD_SECONDS = 2;
/** Width of an export's thumbnail (its height follows the frame's aspect). */
const THUMBNAIL_WIDTH = 320;
/** JPEG quality of the thumbnail. */
const THUMBNAIL_QUALITY = 0.8;

/** What could not be drawn as asked during an export (reported once, not per frame). */
export interface ExportDrawIssues {
  /** Assets whose pictures were missing, still preparing or undecodable. */
  readonly assets: ReadonlySet<string>;
  /** Font families the catalogue lacks (drawn with fallbacks). */
  readonly fonts: ReadonlySet<string>;
  /** Motion templates drawn as the error picture. */
  readonly templates: ReadonlySet<string>;
}

/**
 * Draws the frames of an export in order with the preview's {@link FrameRenderer} (golden
 * rule 7): before each frame the decoder pool's sequential readers are pointed at the next two
 * seconds, exactly like forward playback, then the frame is drawn in playback mode (it waits for
 * its pictures: an export never skips a frame). Originals or mezzanines only, never proxies.
 */
export class ExportFrameDrawer {
  private readonly assets = new Set<string>();
  private readonly fonts = new Set<string>();
  private readonly templates = new Set<string>();

  constructor(
    private readonly renderer: FrameRenderer,
    private readonly pool: DecoderPool,
    private readonly scene: RenderScene,
    private readonly locator: MediaLocator,
  ) {}

  /** Draws the frame at sequence `time` on the renderer's canvas. */
  async draw(time: Flicks): Promise<void> {
    const spans = prefetchSpans(this.scene.sequence, time, 1, READ_AHEAD_SECONDS);
    this.pool.prefetch(
      prefetchRequests(spans, {
        assets: this.scene.assets,
        proxy: false,
        locator: this.locator,
        direction: 1,
        budget: this.pool.readAheadBudget(spans.length),
      }),
    );
    const outcome = await this.renderer.render({
      scene: this.scene,
      time,
      proxy: false,
      playback: true,
    });
    if (outcome.kind !== 'drawn')
      throw new Error('an export frame was replaced before it was drawn');
    for (const issue of outcome.issues) this.assets.add(issue.assetId);
    for (const family of outcome.missingFonts) this.fonts.add(family);
    for (const issue of outcome.templateIssues) this.templates.add(issue.itemId);
  }

  /** What could not be drawn so far. */
  issues(): ExportDrawIssues {
    return { assets: this.assets, fonts: this.fonts, templates: this.templates };
  }

  /** Stops the readers (the export is over). */
  stop(): void {
    this.pool.prefetch([]);
  }
}

/**
 * A reader of `canvas`'s pixels as RGBA bytes (masters go to ffmpeg as raw frames): each call
 * returns a new buffer, so one can upload while the next frame is drawn.
 */
export function rgbaReader(canvas: OffscreenCanvas): () => Uint8Array {
  const copy = new OffscreenCanvas(canvas.width, canvas.height);
  const context = copy.getContext('2d', { willReadFrequently: true });
  if (context === null) throw new Error('this computer cannot read exported frames back');
  return () => {
    context.clearRect(0, 0, copy.width, copy.height);
    context.drawImage(canvas, 0, 0);
    const pixels = context.getImageData(0, 0, copy.width, copy.height).data;
    return new Uint8Array(pixels.buffer, pixels.byteOffset, pixels.byteLength);
  };
}

/** A small JPEG of what `canvas` shows (the exports list's thumbnail). */
export async function thumbnailOf(canvas: OffscreenCanvas): Promise<Blob> {
  const width = Math.min(THUMBNAIL_WIDTH, canvas.width);
  const height = Math.max(2, Math.round((canvas.height * width) / canvas.width));
  const small = new OffscreenCanvas(width, height);
  const context = small.getContext('2d');
  if (context === null) throw new Error('this computer cannot draw thumbnails');
  context.drawImage(canvas, 0, 0, width, height);
  return small.convertToBlob({ type: 'image/jpeg', quality: THUMBNAIL_QUALITY });
}
