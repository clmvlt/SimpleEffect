// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Outcome of the job producing a derived file.
 */
export type DerivedStatus = { "kind": "pending" } | { "kind": "ready" } | { "kind": "failed", 
/**
 * Why, for the user and for agents.
 */
message: string, };
