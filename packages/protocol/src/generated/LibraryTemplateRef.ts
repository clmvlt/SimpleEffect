// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { LibraryTemplateId } from "./LibraryTemplateId";

/**
 * A template of the app's library, named so it can be shown without the library.
 */
export type LibraryTemplateRef = { 
/**
 * The library entry.
 */
id: LibraryTemplateId, 
/**
 * Its name when it was chosen.
 */
name: string, };
