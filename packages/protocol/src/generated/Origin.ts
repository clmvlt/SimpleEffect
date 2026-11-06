// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TaskId } from "./TaskId";

/**
 * Who created an item.
 */
export type Origin = { "kind": "user" } | { "kind": "aiTask", 
/**
 * The task whose changeset contains the creation.
 */
taskId: TaskId, };
