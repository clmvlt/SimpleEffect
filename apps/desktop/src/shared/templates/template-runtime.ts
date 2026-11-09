import type { TemplateRenderer, TemplateRuntime } from '@se/engine/templates';
import {
  assetMediaPath,
  mediaUrl,
  type AssetId,
  type CoreEndpoint,
  type FontFamily,
} from '@se/protocol';

import { catalogFaces } from '../fonts';

/** What the motion template runtime reads its fonts and images from. */
export interface TemplateRuntimeSource {
  readonly endpoint: CoreEndpoint;
  /** The font catalogue (its faces are sent to template Workers as bytes). */
  readonly families: readonly FontFamily[];
}

/** The renderer of the template runtime, plus what a page should free. */
export type AppTemplateRuntime = TemplateRenderer & Pick<TemplateRuntime, 'destroy'>;

let current: { readonly key: string; readonly runtime: Promise<AppTemplateRuntime> } | null = null;

/** Whether this page can run template Workers (not in jsdom). */
export function templatesSupported(): boolean {
  return typeof Worker !== 'undefined' && typeof OffscreenCanvas !== 'undefined';
}

/**
 * The app's one motion template runtime (prompt 24), shared by the viewer, frames for agents and
 * template previews: one Worker per template version for the whole window, one frame cache.
 * `@se/engine/templates` (the compiler) loads the first time. A new endpoint (a restarted core)
 * or font catalogue makes a new runtime and frees the old one. Rejects where Workers or
 * `OffscreenCanvas` are missing.
 */
export function templateRuntimeFor(source: TemplateRuntimeSource): Promise<AppTemplateRuntime> {
  if (!templatesSupported()) {
    return Promise.reject(new Error('motion templates need Workers and OffscreenCanvas'));
  }
  const key = `${source.endpoint.url}|${source.endpoint.token}|${String(source.families.length)}`;
  if (current?.key === key) return current.runtime;
  const previous = current;
  const runtime = createRuntime(source);
  current = { key, runtime };
  void previous?.runtime.then((old) => {
    old.destroy();
  });
  return runtime;
}

async function createRuntime({
  endpoint,
  families,
}: TemplateRuntimeSource): Promise<AppTemplateRuntime> {
  const templates = await import('@se/engine/templates');
  return new templates.TemplateRuntime({
    fonts: templates.catalogFontSource(catalogFaces(families, endpoint)),
    images: (assetId: AssetId) =>
      loadImage(mediaUrl(endpoint, assetMediaPath(assetId, 'original'))),
  });
}

/** An image asset's picture (the Worker draws it), or `null` when it cannot be read. */
async function loadImage(url: string): Promise<ImageBitmap | null> {
  const response = await fetch(url);
  if (!response.ok) return null;
  return createImageBitmap(await response.blob());
}
