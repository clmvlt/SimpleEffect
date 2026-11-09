// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ResetValue } from "./ResetValue";

/**
 * What updating a project template from the library did.
 */
export type TemplateUpdateReport = { 
/**
 * The project template's new version.
 */
version: number, 
/**
 * The library version it now holds.
 */
libraryVersion: number, 
/**
 * Instance values that no longer fitted and went back to the default, as
 * `item id: parameter` pairs.
 */
reset: Array<ResetValue>, };
