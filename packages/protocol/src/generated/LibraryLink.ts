// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { LibraryTemplateId } from "./LibraryTemplateId";

/**
 * The app's template library entry a project template is a copy of, and which version of it.
 *
 * The project keeps the whole template (it opens without the library); the link only lets the
 * interface propose an update when the library holds a newer version (never applied by itself),
 * and lets « Enregistrer dans la bibliothèque » write a new version of the same entry.
 */
export type LibraryLink = { 
/**
 * The library entry.
 */
templateId: LibraryTemplateId, 
/**
 * The library version this copy holds.
 */
version: number, };
