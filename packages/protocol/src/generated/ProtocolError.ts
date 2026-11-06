// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol).
// Regenerate with `pnpm codegen`.
import type { ErrorCode } from "./ErrorCode";
import type { JsonValue } from "./serde_json/JsonValue";

/**
 * An error answering a request. Messages are read by users and AI agents: keep them actionable.
 */
export type ProtocolError = { 
/**
 * Stable, machine-readable error code.
 */
code: ErrorCode, 
/**
 * Human- and agent-readable explanation, including how to fix the problem.
 */
message: string, 
/**
 * Optional structured context (e.g. the unknown command kind). Omitted when absent.
 */
details?: JsonValue, };
