// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { LibraryTemplate } from "./LibraryTemplate";

/**
 * What importing one `.setemplate` file did.
 */
export type LibraryImportOutcome = { "kind": "imported", 
/**
 * The entry.
 */
template: LibraryTemplate, } | { "kind": "updated", 
/**
 * The entry.
 */
template: LibraryTemplate, } | { "kind": "unchanged", 
/**
 * The library's entry.
 */
template: LibraryTemplate, } | { "kind": "failed", 
/**
 * Why, and what to do.
 */
message: string, };
