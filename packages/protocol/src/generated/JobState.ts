// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Where a job stands.
 */
export type JobState = { "kind": "queued" } | { "kind": "running", 
/**
 * Fraction done in `[0, 1]`, when the job can tell.
 */
progress: number | null, } | { "kind": "completed" } | { "kind": "failed", 
/**
 * Why the last attempt failed.
 */
message: string, } | { "kind": "cancelled" };
