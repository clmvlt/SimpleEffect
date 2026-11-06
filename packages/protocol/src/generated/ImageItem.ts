// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { Transform2D } from "./Transform2D";

/**
 * Shows an image asset (a logo, a photo, a sticker) for the item's duration.
 */
export type ImageItem = { 
/**
 * The image asset (its kind must be `image`).
 */
assetId: AssetId, 
/**
 * Placement of the image.
 */
transform: Transform2D, };
