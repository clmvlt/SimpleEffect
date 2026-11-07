// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { ItemEdge } from "./ItemEdge";
import type { ItemId } from "./ItemId";

/**
 * One edge of an item moved (`trimItems`).
 */
export type ItemTrim = { 
/**
 * The item trimmed.
 */
itemId: ItemId, 
/**
 * The edge moved.
 */
edge: ItemEdge, 
/**
 * The new position of that edge in the sequence.
 */
to: Flicks, };
