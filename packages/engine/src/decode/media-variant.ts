import type { Asset, AssetId } from '@se/protocol';

import { playbackSource } from '../sources/playback-source';

/** Which file of an asset is decoded (`docs/architecture.md` §7). */
export type MediaVariant = 'original' | 'mezzanine' | 'proxy';

/** The file to decode for an asset, or why there is none yet. */
export type VariantChoice =
  | { readonly kind: 'ready'; readonly variant: MediaVariant }
  /** Decodability check or mezzanine still running. */
  | { readonly kind: 'preparing' }
  | { readonly kind: 'unavailable'; readonly reason: 'missing' | 'mezzanineFailed' };

/** Viewer options that change the decoded file. */
export interface VariantOptions {
  /** Prefer the low-resolution proxy when one is ready (the viewer's « Proxy » toggle). */
  readonly proxy: boolean;
}

/**
 * Chooses the file to decode: the ready proxy when the option is on (even if the original went
 * offline), otherwise what `playbackSource` says (the original when decodable, else the
 * mezzanine). Still images always use their original.
 */
export function chooseVariant(asset: Asset, options: VariantOptions): VariantChoice {
  if (asset.kind === 'image') {
    return asset.missing
      ? { kind: 'unavailable', reason: 'missing' }
      : { kind: 'ready', variant: 'original' };
  }
  if (options.proxy && asset.derived.proxy?.status.kind === 'ready') {
    return { kind: 'ready', variant: 'proxy' };
  }
  const source = playbackSource(asset);
  switch (source.kind) {
    case 'original':
    case 'mezzanine':
      return { kind: 'ready', variant: source.kind };
    case 'preparing':
      return source;
    case 'unavailable':
      return source;
  }
}

/**
 * Where the engine reads an asset's files: se-core's `/media` URLs (with the session token).
 * `null` while no core is connected. URLs must change when the file's content changes (versioned).
 */
export type MediaLocator = (assetId: AssetId, variant: MediaVariant) => string | null;
