// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Stable error codes (`SCREAMING_SNAKE_CASE` on the wire). Never rename or reuse a code: clients
 * and AI agents match on them. Later prompts add codes for their own failures.
 */
export type ErrorCode = "INVALID_REQUEST" | "UNKNOWN_COMMAND" | "INTERNAL" | "TIMEOUT" | "CONNECTION_LOST" | "CLIENT_CLOSED";
