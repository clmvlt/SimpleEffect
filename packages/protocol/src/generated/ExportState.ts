// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ExportCheck } from "./ExportCheck";

/**
 * Where an export stands.
 */
export type ExportState = { "kind": "queued" } | { "kind": "rendering", 
/**
 * Fraction done in `[0, 1]`.
 */
progress: number, 
/**
 * Seconds left, once they can be estimated.
 */
etaSeconds: number | null, } | { "kind": "finalizing" } | { "kind": "completed", 
/**
 * Its size in bytes.
 */
sizeBytes: number, 
/**
 * Seconds from the start of the render to the end.
 */
elapsedSeconds: number, 
/**
 * What ffprobe found, `null` when it could not read the file.
 */
check: ExportCheck | null, } | { "kind": "failed", 
/**
 * Why (actionable).
 */
message: string, } | { "kind": "cancelled" };
