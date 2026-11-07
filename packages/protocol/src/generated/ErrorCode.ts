// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Stable error codes (`SCREAMING_SNAKE_CASE` on the wire). Never rename or reuse a code: clients
 * and AI agents match on them. Later prompts add codes for their own failures.
 */
export type ErrorCode = "INVALID_REQUEST" | "UNKNOWN_COMMAND" | "INTERNAL" | "TIMEOUT" | "CONNECTION_LOST" | "CLIENT_CLOSED" | "PROJECT_NOT_FOUND" | "PROJECT_LOCKED" | "SCHEMA_TOO_NEW" | "PROJECT_CORRUPTED" | "PROJECT_ALREADY_EXISTS" | "PROJECT_NOT_OPEN" | "STORAGE_FAILED" | "EDIT_REJECTED" | "NOTHING_TO_UNDO" | "NOTHING_TO_REDO" | "ASSET_NOT_FOUND" | "MEDIA_NOT_FOUND" | "MEDIA_UNSUPPORTED" | "MEDIA_KIND_MISMATCH" | "MEDIA_TOOLS_MISSING" | "JOB_NOT_FOUND" | "DERIVED_NOT_APPLICABLE" | "MEDIA_TIME_OUT_OF_RANGE" | "RIPPLE_BLOCKED" | "TRANSITION_NO_ROOM";
