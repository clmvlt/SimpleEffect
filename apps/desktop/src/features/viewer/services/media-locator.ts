import {
  assetMediaPath,
  mediaUrl,
  type Asset,
  type AssetId,
  type CoreEndpoint,
} from '@se/protocol';
import type { MediaLocator, MediaVariant } from '@se/engine/preview';

/**
 * Version of a file in its URL: the engine reopens a decoder when the URL changes, so it changes
 * with the content (relinked original, regenerated mezzanine or proxy at another path).
 */
function fileVersion(asset: Asset, variant: MediaVariant): string {
  const content = asset.fingerprint?.sampleHash ?? '';
  switch (variant) {
    case 'original':
      return content;
    case 'mezzanine':
      return `${content}:${asset.derived.mezzanine?.relativePath ?? ''}`;
    case 'proxy':
      return `${content}:${asset.derived.proxy?.relativePath ?? ''}`;
  }
}

/**
 * The engine's media locator over se-core's `/media` routes (`docs/architecture.md` §7): `null`
 * while no core is connected or for an unknown asset. URLs carry the session token: never log them.
 */
export function createMediaLocator(
  endpoint: CoreEndpoint | null,
  assets: Readonly<Record<AssetId, Asset>>,
): MediaLocator {
  return (assetId, variant) => {
    const asset = assets[assetId];
    if (endpoint === null || asset === undefined) return null;
    return mediaUrl(endpoint, assetMediaPath(assetId, variant), fileVersion(asset, variant));
  };
}
