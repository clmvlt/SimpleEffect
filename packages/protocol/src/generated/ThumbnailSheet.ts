// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * Layout of an asset's thumbnail sheet, served as JSON by `/media/assets/{id}/thumbnails`; the
 * sheet itself (JPEG) is `/media/assets/{id}/thumbnails/sheet` and the poster
 * `/media/assets/{id}/thumbnails/poster`.
 *
 * Tiles are laid out left to right, then top to bottom; tile `i` shows the picture at `times[i]`.
 */
export type ThumbnailSheet = { 
/**
 * Width of a tile in pixels.
 */
tileWidth: number, 
/**
 * Height of a tile in pixels (90).
 */
tileHeight: number, 
/**
 * Tiles per row.
 */
columns: number, 
/**
 * Rows of tiles.
 */
rows: number, 
/**
 * Source time of each tile (`times.len()` tiles; the last row may be incomplete).
 */
times: Array<Flicks>, 
/**
 * Width of the poster in pixels.
 */
posterWidth: number, 
/**
 * Height of the poster in pixels.
 */
posterHeight: number, };
