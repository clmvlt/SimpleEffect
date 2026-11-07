// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { ShapeKind } from "./ShapeKind";

/**
 * What a new graphic item shows (`addGraphicItem`). It starts with the default look (white
 * Inter text, white shape, image fitted into the frame), centred.
 */
export type NewGraphic = { "kind": "text", 
/**
 * Its content (lines separated by `\n`).
 */
content: string, } | { "kind": "shape", 
/**
 * Which shape.
 */
shape: ShapeKind, } | { "kind": "image", 
/**
 * The image asset.
 */
assetId: AssetId, };
