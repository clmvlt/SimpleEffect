// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";

/**
 * A media whose faces could not be analysed (`autoReframeFinished`).
 */
export type ReframeFailure = { 
/**
 * The media.
 */
assetId: AssetId, 
/**
 * Why (actionable).
 */
message: string, };
