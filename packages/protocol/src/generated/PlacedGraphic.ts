// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { TrackId } from "./TrackId";

/**
 * Where `addGraphicItem` put the new item.
 */
export type PlacedGraphic = { 
/**
 * The new item.
 */
itemId: ItemId, 
/**
 * Its track (possibly a new one).
 */
trackId: TrackId, };
