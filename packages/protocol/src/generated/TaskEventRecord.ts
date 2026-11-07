// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AgentEvent } from "./AgentEvent";

/**
 * One entry of a task's event log.
 */
export type TaskEventRecord = { 
/**
 * Position in the task's log, from 0.
 */
sequence: number, 
/**
 * Index of the run that produced it in [`Task::runs`].
 */
run: number, 
/**
 * When it was received (milliseconds since the Unix epoch).
 */
at: number, 
/**
 * What the agent reported.
 */
event: AgentEvent, };
