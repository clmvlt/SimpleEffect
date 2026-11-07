// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ErrorCode } from "./ErrorCode";

/**
 * A path that could not be imported.
 */
export type ImportFailure = { 
/**
 * The path given.
 */
path: string, 
/**
 * Why (`MEDIA_NOT_FOUND`, `MEDIA_UNSUPPORTED`, `MEDIA_TOOLS_MISSING`…).
 */
code: ErrorCode, 
/**
 * Actionable explanation.
 */
message: string, };
