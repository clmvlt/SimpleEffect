// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Format } from "./Format";
import type { MarkerId } from "./MarkerId";
import type { SequenceId } from "./SequenceId";
import type { TrackId } from "./TrackId";

/**
 * A sequence without its tracks and markers, which are entities of their own.
 */
export type SequenceEntity = { 
/**
 * Stable id.
 */
id: SequenceId, 
/**
 * Name shown in the UI.
 */
name: string, 
/**
 * Frame size and rates.
 */
format: Format, 
/**
 * Its tracks, in stacking order (first video track = bottom layer).
 */
trackIds: Array<TrackId>, 
/**
 * Its markers, in document order.
 */
markerIds: Array<MarkerId>, };
