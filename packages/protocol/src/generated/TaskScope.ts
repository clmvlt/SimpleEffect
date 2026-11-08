// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { SequenceId } from "./SequenceId";
import type { TimeRange } from "./TimeRange";
import type { TrackId } from "./TrackId";

/**
 * What part of the project a task may change (checked by MCP write tools, prompt 19).
 */
export type TaskScope = { 
/**
 * The sequence the task works on.
 */
sequenceId: SequenceId, 
/**
 * The time range it may change; `null` for the whole sequence.
 */
range: TimeRange | null, 
/**
 * The tracks it may change; `null` for every track.
 */
tracks: Array<TrackId> | null, };
