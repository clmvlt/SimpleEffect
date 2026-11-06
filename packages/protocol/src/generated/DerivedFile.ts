// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DerivedStatus } from "./DerivedStatus";

/**
 * One generated file and the outcome of its job.
 */
export type DerivedFile = { 
/**
 * Location relative to the project folder, `/`-separated.
 */
relativePath: string, 
/**
 * Whether the file can be used. Job progress is runtime state, not stored here.
 */
status: DerivedStatus, };
