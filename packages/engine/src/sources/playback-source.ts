import type { Asset, DerivedFile } from '@se/protocol';

/**
 * Which file the engine decodes for an asset (`docs/architecture.md` §7):
 * - `original`: WebCodecs decodes the user's file (or it is a still `ImageDecoder` reads);
 * - `mezzanine`: the original is not decodable here, its mezzanine is ready;
 * - `preparing`: the original is not decodable and its mezzanine is queued or running (or the
 *   decodability check is still running): the asset cannot be played yet;
 * - `unavailable`: the original is missing, or not decodable and its mezzanine failed.
 */
export type PlaybackSource =
  | { readonly kind: 'original' }
  | { readonly kind: 'mezzanine'; readonly relativePath: string }
  | { readonly kind: 'preparing' }
  | { readonly kind: 'unavailable'; readonly reason: 'missing' | 'mezzanineFailed' };

/** Whether a stream needs the mezzanine: WebCodecs refused it. */
function refused(state: Asset['decodability']['video']): boolean {
  return state === 'undecodable';
}

function fromMezzanine(mezzanine: DerivedFile | null): PlaybackSource {
  switch (mezzanine?.status.kind) {
    case 'ready':
      return { kind: 'mezzanine', relativePath: mezzanine.relativePath };
    case 'failed':
      return { kind: 'unavailable', reason: 'mezzanineFailed' };
    case 'pending':
    case undefined:
      return { kind: 'preparing' };
  }
}

/**
 * The file to decode for `asset`. A ready mezzanine is used whenever one exists (it also covers
 * 4:2:2 and 12-bit footage the browser reports decodable but renders badly, so a pending one is
 * waited for); otherwise the original plays once no stream was refused.
 */
export function playbackSource(asset: Asset): PlaybackSource {
  if (asset.missing) {
    return { kind: 'unavailable', reason: 'missing' };
  }
  const { video, audio } = asset.decodability;
  const mezzanine = asset.derived.mezzanine;
  if (mezzanine?.status.kind === 'ready' || refused(video) || refused(audio)) {
    return fromMezzanine(mezzanine);
  }
  if (video === 'unknown' || audio === 'unknown' || mezzanine?.status.kind === 'pending') {
    return { kind: 'preparing' };
  }
  return { kind: 'original' };
}

/** Whether the asset can be played now (from its original or its mezzanine). */
export function isPlayable(asset: Asset): boolean {
  const source = playbackSource(asset);
  return source.kind === 'original' || source.kind === 'mezzanine';
}
