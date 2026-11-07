import type { Flicks, ThumbnailSheet } from '@se/protocol';

/** The tile of a thumbnail sheet nearest a source time, as a CSS background of a box. */
export interface TileBackground {
  readonly size: string;
  readonly position: string;
}

/** Index of the tile whose time is nearest `sourceTime` (−1 for an empty sheet). */
export function nearestTile(sheet: ThumbnailSheet, sourceTime: Flicks): number {
  let best = -1;
  let distance = Number.POSITIVE_INFINITY;
  sheet.times.forEach((time, index) => {
    const gap = Math.abs(time - sourceTime);
    if (gap < distance) {
      best = index;
      distance = gap;
    }
  });
  return best;
}

/** The background showing tile `index` stretched over a box of `width` × `height` pixels. */
export function tileBackground(
  sheet: ThumbnailSheet,
  index: number,
  width: number,
  height: number,
): TileBackground {
  const column = index % sheet.columns;
  const row = Math.floor(index / sheet.columns);
  const scaleX = width / sheet.tileWidth;
  const scaleY = height / sheet.tileHeight;
  return {
    size: `${String(sheet.columns * sheet.tileWidth * scaleX)}px ${String(sheet.rows * sheet.tileHeight * scaleY)}px`,
    position: `${String(-column * width)}px ${String(-row * height)}px`,
  };
}
