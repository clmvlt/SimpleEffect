// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";

/**
 * A path that was not imported because the project already has its content.
 */
export type DuplicateMedia = { 
/**
 * The path given.
 */
path: string, 
/**
 * The asset holding the same content.
 */
assetId: AssetId, };
