// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The whisper.cpp program found (or why none).
 */
export type WhisperInfo = { 
/**
 * Absolute path of the `whisper-cli` used, when found.
 */
path: string | null, 
/**
 * What is wrong, when it cannot run (not found, missing system libraries).
 */
problem: string | null, };
