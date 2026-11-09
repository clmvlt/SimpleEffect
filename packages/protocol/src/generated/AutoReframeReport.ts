// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { ItemId } from "./ItemId";
import type { SkippedReframe } from "./SkippedReframe";

/**
 * What automatic reframing did with the clips it was given (`autoReframeItems`, and versions
 * made with automatic reframing).
 */
export type AutoReframeReport = { 
/**
 * Clips reframed now (their faces were already analysed), in one undoable step.
 */
appliedItemIds: Array<ItemId>, 
/**
 * Media whose faces are being analysed: their clips are reframed when the analysis ends
 * (`autoReframeFinished`), in one more undoable step.
 */
waitingAssetIds: Array<AssetId>, 
/**
 * Clips left as they are, and why.
 */
skipped: Array<SkippedReframe>, };
