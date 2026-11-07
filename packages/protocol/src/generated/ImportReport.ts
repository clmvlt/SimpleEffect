// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { DuplicateMedia } from "./DuplicateMedia";
import type { ImportFailure } from "./ImportFailure";

/**
 * What [`crate::Command::ImportMedia`] did with each path.
 */
export type ImportReport = { 
/**
 * Assets created (one undoable transaction), in the order of the paths.
 */
imported: Array<AssetId>, 
/**
 * Paths whose content is already in the project (or earlier in the same request).
 */
duplicates: Array<DuplicateMedia>, 
/**
 * Paths that could not be imported, with the reason.
 */
failures: Array<ImportFailure>, };
