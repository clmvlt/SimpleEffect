// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionStylePatch } from "./CaptionStylePatch";
import type { ItemId } from "./ItemId";
import type { ProjectId } from "./ProjectId";

/**
 * A change of the style of captions, made only for them (copy on write).
 */
export type CaptionStyleEdit = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The captions (all those of a track to change the track's look). A style also used by
 * captions not listed is copied for the listed ones.
 */
itemIds: Array<ItemId>, 
/**
 * What changes.
 */
patch: CaptionStylePatch, };
