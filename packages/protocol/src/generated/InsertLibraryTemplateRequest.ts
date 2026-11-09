// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { LibraryTemplateId } from "./LibraryTemplateId";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";
import type { TrackId } from "./TrackId";

/**
 * Where a library template is inserted in a project.
 */
export type InsertLibraryTemplateRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The library entry (a motion template).
 */
templateId: LibraryTemplateId, 
/**
 * The sequence.
 */
sequenceId: SequenceId, 
/**
 * The video track (`null`: the first free video track at that time, or a new one).
 */
trackId: TrackId | null, 
/**
 * Where the instance starts.
 */
start: Flicks, };
