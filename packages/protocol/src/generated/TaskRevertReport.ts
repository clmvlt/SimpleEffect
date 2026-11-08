// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Conflict } from "./Conflict";
import type { ItemId } from "./ItemId";
import type { TaskId } from "./TaskId";

/**
 * What reverting an AI task did ([`crate::Command::RevertTask`]).
 */
export type TaskRevertReport = { 
/**
 * The task.
 */
taskId: TaskId, 
/**
 * Whether anything was undone: the revert is then one user transaction (undoable), and
 * the task's changeset is closed.
 */
reverted: boolean, 
/**
 * Items the revert restored, removed or put back.
 */
revertedItems: Array<ItemId>, 
/**
 * What stays as the task left it, and why (items changed by someone else since, edits that
 * no longer apply).
 */
conflicts: Array<Conflict>, };
