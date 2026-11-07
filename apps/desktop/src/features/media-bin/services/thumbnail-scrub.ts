import type { ThumbnailSheet } from '@se/protocol';

/** The tile under a pointer at `fraction` (0 = left edge, 1 = right edge) of the thumbnail. */
export function tileIndexAt(fraction: number, tileCount: number): number {
  if (tileCount <= 1 || !Number.isFinite(fraction)) {
    return 0;
  }
  const clamped = Math.min(Math.max(fraction, 0), 1);
  return Math.min(Math.floor(clamped * tileCount), tileCount - 1);
}

/**
 * CSS background values showing tile `index` of the sheet scaled to `width` pixels: the sheet
 * image is the background, sized so one tile fills the box.
 */
export function tileBackground(
  sheet: ThumbnailSheet,
  index: number,
  width: number,
): { size: string; position: string; height: number } {
  const scale = width / sheet.tileWidth;
  const column = index % sheet.columns;
  const row = Math.floor(index / sheet.columns);
  return {
    size: `${String(sheet.columns * width)}px ${String(sheet.rows * sheet.tileHeight * scale)}px`,
    position: `-${String(column * width)}px -${String(row * sheet.tileHeight * scale)}px`,
    height: sheet.tileHeight * scale,
  };
}
