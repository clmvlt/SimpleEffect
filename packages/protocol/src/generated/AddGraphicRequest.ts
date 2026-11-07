// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { NewGraphic } from "./NewGraphic";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";

/**
 * A graphic item to add at a time (`addGraphicItem`): it goes on the highest unlocked video
 * track free over its span with nothing above it at that time, or on a new video track on top.
 */
export type AddGraphicRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence receiving it.
 */
sequenceId: SequenceId, 
/**
 * Where it starts.
 */
at: Flicks, 
/**
 * How long it lasts (> 0).
 */
duration: Flicks, 
/**
 * What it shows.
 */
graphic: NewGraphic, };
