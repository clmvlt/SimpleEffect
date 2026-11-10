import type { Asset, AssetId } from '@se/protocol';

import type { MediaLocator } from '../decode/media-variant';
import type { AssetTable } from '../evaluate/item-layer';
import { playbackSource } from '../sources/playback-source';

/** Whether `asset` may have sound to play (images never do). */
function hasSound(asset: Asset): boolean {
  return asset.kind !== 'image' && asset.decodability.audio !== 'noStream';
}

/**
 * The file whose sound `assetId` plays: its original, or its mezzanine (never the proxy); `null`
 * when it has no sound to play now. Playback and export read the same file (golden rule 7).
 */
export function soundFileOf(
  assets: AssetTable,
  locator: MediaLocator,
  assetId: AssetId,
): string | null {
  const asset = assets[assetId];
  if (asset === undefined || !hasSound(asset)) return null;
  const source = playbackSource(asset);
  if (source.kind !== 'original' && source.kind !== 'mezzanine') return null;
  return locator(assetId, source.kind);
}
