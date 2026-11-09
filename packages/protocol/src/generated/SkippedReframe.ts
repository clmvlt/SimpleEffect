// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { ReframeSkipReason } from "./ReframeSkipReason";

/**
 * A clip automatic reframing cannot follow.
 */
export type SkippedReframe = { 
/**
 * The item.
 */
itemId: ItemId, 
/**
 * Why it was left.
 */
reason: ReframeSkipReason, };
