// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Tokens and cost of a task's runs, added up from their `usage` events.
 */
export type TaskUsage = { 
/**
 * Input tokens (cached ones included).
 */
inputTokens: number, 
/**
 * Output tokens.
 */
outputTokens: number, 
/**
 * Cost in US dollars of the runs whose CLI reported one; `null` when none did.
 */
costUsd: number | null, };
