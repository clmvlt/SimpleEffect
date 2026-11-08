// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TaskId } from "./TaskId";

/**
 * Who performs a transaction.
 */
export type Actor = { "kind": "user" } | { "kind": "aiTask", 
/**
 * The task.
 */
taskId: TaskId, } | { "kind": "system" };
