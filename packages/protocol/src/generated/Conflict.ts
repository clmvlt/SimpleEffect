// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DomainErrorCode } from "./DomainErrorCode";
import type { ItemId } from "./ItemId";

/**
 * A part of a task's changes that a revert leaves in place.
 */
export type Conflict = { "kind": "itemChanged", 
/**
 * The item.
 */
item: ItemId, 
/**
 * Its revision right after the task's last edit (`null`: the task removed it).
 */
taskRevision: number | null, 
/**
 * Its current revision (`null`: it no longer exists).
 */
currentRevision: number | null, } | { "kind": "foreignItems", 
/**
 * The items that would have been affected.
 */
items: Array<ItemId>, } | { "kind": "cannotApply", 
/**
 * Why, as a stable code.
 */
code: DomainErrorCode, 
/**
 * Why, for people and agents.
 */
message: string, };
