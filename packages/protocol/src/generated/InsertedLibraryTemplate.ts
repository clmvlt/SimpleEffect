// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { TemplateId } from "./TemplateId";
import type { TrackId } from "./TrackId";

/**
 * What inserting a library template did.
 */
export type InsertedLibraryTemplate = { 
/**
 * The project's copy of the template (an existing copy of the same version is reused).
 */
templateId: TemplateId, 
/**
 * The new instance.
 */
itemId: ItemId, 
/**
 * Its track.
 */
trackId: TrackId, };
