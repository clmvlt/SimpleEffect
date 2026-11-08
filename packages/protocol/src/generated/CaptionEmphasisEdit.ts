// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { ProjectId } from "./ProjectId";

/**
 * Emphasis of some words of a caption.
 */
export type CaptionEmphasisEdit = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The caption.
 */
itemId: ItemId, 
/**
 * Indices of its words (0-based).
 */
wordIndices: Array<number>, 
/**
 * Put forward (`true`) or back to normal.
 */
emphasis: boolean, };
