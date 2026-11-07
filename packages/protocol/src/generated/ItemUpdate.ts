// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { ItemPatch } from "./ItemPatch";

/**
 * A typed change of one item (`updateItems`).
 */
export type ItemUpdate = { 
/**
 * The item changed.
 */
itemId: ItemId, 
/**
 * What changes; its kind must match the item's.
 */
patch: ItemPatch, };
