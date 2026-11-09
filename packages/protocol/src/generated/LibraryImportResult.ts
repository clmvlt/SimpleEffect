// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { LibraryImportOutcome } from "./LibraryImportOutcome";

/**
 * The outcome of one file of an import.
 */
export type LibraryImportResult = { 
/**
 * The file.
 */
path: string, 
/**
 * What happened.
 */
outcome: LibraryImportOutcome, };
