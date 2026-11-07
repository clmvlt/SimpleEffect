// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { RunKind } from "./RunKind";
import type { TaskStatus } from "./TaskStatus";

/**
 * One execution of the agent for a task.
 */
export type TaskRun = { 
/**
 * Why it was started.
 */
runKind: RunKind, 
/**
 * The prompt it sent.
 */
prompt: string, 
/**
 * Where it stands.
 */
status: TaskStatus, 
/**
 * When it was queued (milliseconds since the Unix epoch).
 */
queuedAt: number, 
/**
 * When the agent was started.
 */
startedAt: number | null, 
/**
 * When it ended.
 */
finishedAt: number | null, };
