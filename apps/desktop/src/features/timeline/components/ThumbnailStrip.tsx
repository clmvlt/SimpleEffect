import type { Asset, Flicks, ThumbnailSheet } from '@se/protocol';

import { useAssetMediaUrl } from '../../media-bin';
import { useCachedResource } from '../hooks/use-cached-resource';
import { stripTiles, tileBackground } from '../services/thumbnail-strip';

/** Props of {@link ThumbnailStrip}. */
export interface ThumbnailStripProps {
  readonly asset: Asset;
  readonly sourceStart: Flicks;
  readonly speed: number;
  readonly pixelsPerSecond: number;
  /** Pixels of the clip hidden left of the strip. */
  readonly offsetPixels: number;
  readonly width: number;
  readonly height: number;
}

const readSheet = (response: Response) => response.json() as Promise<ThumbnailSheet>;

/**
 * Thumbnails along the visible part of a video clip, cut from the asset's thumbnail sheet
 * (prompt 08): each tile is the sheet image as a background, sized and offset to show one tile.
 */
export function ThumbnailStrip({
  asset,
  sourceStart,
  speed,
  pixelsPerSecond,
  offsetPixels,
  width,
  height,
}: ThumbnailStripProps) {
  const layoutUrl = useAssetMediaUrl(asset, 'thumbnails', 'thumbnails');
  const sheetUrl = useAssetMediaUrl(asset, 'thumbnailSheet', 'thumbnails');
  const sheet = useCachedResource(layoutUrl, readSheet);
  if (sheet === null || sheetUrl === null) return null;
  const tiles = stripTiles(sheet, {
    sourceStart,
    speed,
    pixelsPerSecond,
    offsetPixels,
    width,
    height,
  });
  return (
    <div aria-hidden className="pointer-events-none absolute inset-0 overflow-hidden">
      {tiles.map((tile) => {
        const background = tileBackground(sheet, tile, height);
        return (
          <div
            key={tile.left}
            className="absolute top-0 bg-no-repeat"
            style={{
              left: tile.left,
              width: tile.width,
              height,
              backgroundImage: `url("${sheetUrl}")`,
              backgroundSize: background.size,
              backgroundPosition: background.position,
            }}
          />
        );
      })}
    </div>
  );
}
