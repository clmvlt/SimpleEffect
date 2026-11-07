import type { Flicks, ThumbnailSheet } from '@se/protocol';

/** What part of a clip a thumbnail strip covers. */
export interface StripWindow {
  /** Source time at the clip's start. */
  readonly sourceStart: Flicks;
  /** Playback speed of the clip. */
  readonly speed: number;
  /** Zoom of the timeline. */
  readonly pixelsPerSecond: number;
  /** Pixels of the clip hidden left of the strip. */
  readonly offsetPixels: number;
  /** Width of the strip in pixels. */
  readonly width: number;
  /** Height of the tiles in pixels (the lane's height). */
  readonly height: number;
}

/** One tile of the strip: where it sits and which tile of the sheet it shows. */
export interface StripTile {
  /** Left edge in the strip, in pixels (the first tile may start before 0). */
  readonly left: number;
  readonly width: number;
  /** Index of the tile in the sheet. */
  readonly index: number;
}

const FLICKS_PER_SECOND = 705_600_000;

/** Index of the last tile taken at or before `time` (tile times are sorted). */
function tileAt(times: readonly Flicks[], time: number): number {
  let low = 0;
  let high = times.length - 1;
  while (low < high) {
    const middle = (low + high + 1) >>> 1;
    if ((times[middle] ?? 0) <= time) low = middle;
    else high = middle - 1;
  }
  return Math.max(0, low);
}

/**
 * The tiles to draw along a clip: tiles keep the sheet's aspect ratio at the lane's height and
 * line up from the clip's start; each shows the sheet tile taken closest before the source time at
 * its left edge. Only tiles crossing the strip's window are returned.
 */
export function stripTiles(sheet: ThumbnailSheet, window: StripWindow): StripTile[] {
  if (sheet.times.length === 0 || window.height <= 0 || window.width <= 0) return [];
  const tileWidth = (sheet.tileWidth / sheet.tileHeight) * window.height;
  const first = Math.floor(window.offsetPixels / tileWidth);
  const last = Math.ceil((window.offsetPixels + window.width) / tileWidth);
  const tiles: StripTile[] = [];
  for (let slot = first; slot < last; slot += 1) {
    const clipSeconds = (slot * tileWidth) / window.pixelsPerSecond;
    const sourceTime = window.sourceStart + clipSeconds * window.speed * FLICKS_PER_SECOND;
    tiles.push({
      left: slot * tileWidth - window.offsetPixels,
      width: tileWidth,
      index: tileAt(sheet.times, sourceTime),
    });
  }
  return tiles;
}

/** CSS background size and position showing `tile` from the sheet image at `height` pixels. */
export function tileBackground(
  sheet: ThumbnailSheet,
  tile: StripTile,
  height: number,
): { readonly size: string; readonly position: string } {
  const scale = height / sheet.tileHeight;
  const column = tile.index % sheet.columns;
  const row = Math.floor(tile.index / sheet.columns);
  return {
    size: `${String(sheet.columns * tile.width)}px ${String(sheet.rows * height)}px`,
    position: `-${String(column * tile.width)}px -${String(row * sheet.tileHeight * scale)}px`,
  };
}
