// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AgentFailureKind } from "./AgentFailureKind";

/**
 * What an agent run reports, in order. A run ends with exactly one [`AgentEvent::Completed`]
 * or [`AgentEvent::Failed`].
 */
export type AgentEvent = { "kind": "sessionStarted", 
/**
 * The CLI's session id.
 */
sessionId: string, 
/**
 * The model it runs, when it says.
 */
model: string | null, } | { "kind": "assistantText", 
/**
 * The text (markdown).
 */
text: string, } | { "kind": "toolCallStarted", 
/**
 * Id pairing the call with its result.
 */
callId: string, 
/**
 * The tool's name.
 */
tool: string, 
/**
 * Its input, shortened for display.
 */
inputSummary: string, } | { "kind": "toolCallFinished", 
/**
 * Id of the call.
 */
callId: string, 
/**
 * The tool's name (empty when the call was not seen).
 */
tool: string, 
/**
 * Whether it succeeded.
 */
ok: boolean, 
/**
 * Its output, shortened for display.
 */
outputSummary: string, } | { "kind": "usage", 
/**
 * Input tokens (cached ones included).
 */
inputTokens: number, 
/**
 * Output tokens.
 */
outputTokens: number, 
/**
 * Cost in US dollars, when the CLI reports it.
 */
costUsd: number | null, 
/**
 * Duration of the run in milliseconds, when the CLI reports it.
 */
durationMs: number | null, } | { "kind": "completed", 
/**
 * The agent's final answer.
 */
summary: string, } | { "kind": "failed", 
/**
 * Why.
 */
failure: AgentFailureKind, 
/**
 * What happened, actionable.
 */
message: string, };
