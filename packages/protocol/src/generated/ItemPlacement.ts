// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { ItemId } from "./ItemId";
import type { TrackId } from "./TrackId";

/**
 * Where an existing item goes (`moveItems`) or where its copy goes (`duplicateItems`).
 */
export type ItemPlacement = { 
/**
 * The item moved or copied.
 */
itemId: ItemId, 
/**
 * The track it (or its copy) ends up on, in the item's sequence.
 */
trackId: TrackId, 
/**
 * Its new start (≥ 0).
 */
start: Flicks, };
