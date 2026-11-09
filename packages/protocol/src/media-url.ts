import type { CoreEndpoint } from './core-endpoint';

/**
 * Samples per peak of the waveform levels se-core writes (`se_media::peaks::PEAK_LEVELS`): each
 * level is 4× coarser. Keep in sync with the Rust constant.
 */
export const PEAK_LEVELS = [256, 1024, 4096] as const;

/** One of {@link PEAK_LEVELS}. */
export type PeakLevel = (typeof PEAK_LEVELS)[number];

/**
 * A file served by se-core's `/media` routes for an asset (`docs/architecture.md` §7):
 * the original, its mezzanine or proxy (once ready), the thumbnail sheet layout (JSON
 * `ThumbnailSheet`), the sheet (JPEG), the poster (JPEG), or waveform peaks (`SEPK`, see
 * `decodeWaveformPeaks`).
 */
export type AssetMediaFile =
  | 'original'
  | 'mezzanine'
  | 'proxy'
  | 'thumbnails'
  | 'thumbnailSheet'
  | 'poster'
  | { readonly peaks: PeakLevel };

/** Path of an asset's media file, e.g. `/media/assets/<id>/thumbnails/sheet`. */
export function assetMediaPath(assetId: string, file: AssetMediaFile): string {
  const base = `/media/assets/${encodeURIComponent(assetId)}`;
  if (typeof file === 'object') {
    return `${base}/peaks/${String(file.peaks)}`;
  }
  switch (file) {
    case 'original':
      return `${base}/original`;
    case 'mezzanine':
      return `${base}/mezzanine`;
    case 'proxy':
      return `${base}/proxy`;
    case 'thumbnails':
      return `${base}/thumbnails`;
    case 'thumbnailSheet':
      return `${base}/thumbnails/sheet`;
    case 'poster':
      return `${base}/thumbnails/poster`;
  }
}

/** Path of a font face of the catalogue (`listFonts`), e.g. `/media/fonts/bundled-inter-latin-wght-normal`. */
export function fontMediaPath(faceId: string): string {
  return `/media/fonts/${encodeURIComponent(faceId)}`;
}

/**
 * Path of a logo of a library brand kit (prompt 25): `file` is the kit's `logos/<name>`, e.g.
 * `/media/brand-kits/<kit id>/logos/<name>.png`.
 */
export function brandKitLogoPath(kitId: string, file: string): string {
  const name = file.startsWith('logos/') ? file.slice('logos/'.length) : file;
  return `/media/brand-kits/${encodeURIComponent(kitId)}/logos/${encodeURIComponent(name)}`;
}

/**
 * Absolute URL of a `/media` path on `endpoint`, with the session token as `t` (media elements
 * and `<img>` cannot send headers). `version` is appended as `v` to defeat caches when the file
 * changes (e.g. after its job completes again). Never log the result: it carries the token.
 */
export function mediaUrl(endpoint: CoreEndpoint, path: string, version?: string | number): string {
  const url = new URL(path, endpoint.url);
  url.searchParams.set('t', endpoint.token);
  if (version !== undefined) {
    url.searchParams.set('v', String(version));
  }
  return url.toString();
}
