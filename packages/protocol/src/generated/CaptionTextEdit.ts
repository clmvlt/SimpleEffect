// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { ProjectId } from "./ProjectId";

/**
 * New words of one caption.
 */
export type CaptionTextEdit = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The caption.
 */
itemId: ItemId, 
/**
 * Its new text: unchanged words keep their timing, changed runs are redistributed.
 */
text: string, };
