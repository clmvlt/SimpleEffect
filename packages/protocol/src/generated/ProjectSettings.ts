// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Format } from "./Format";
import type { TranscriptionSettings } from "./TranscriptionSettings";

/**
 * Project-wide preferences.
 */
export type ProjectSettings = { 
/**
 * Format given to new sequences.
 */
defaultFormat: Format, 
/**
 * How the project's media are transcribed (schema version 7).
 */
transcription: TranscriptionSettings, };
