// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AgentFailureKind } from "./AgentFailureKind";

/**
 * Where a task (or one of its runs) stands: `Queued → Running → Completed | Failed | Cancelled`.
 */
export type TaskStatus = { "kind": "queued" } | { "kind": "running" } | { "kind": "completed" } | { "kind": "failed", 
/**
 * Why.
 */
failure: AgentFailureKind, 
/**
 * What happened, actionable.
 */
message: string, } | { "kind": "cancelled" };
