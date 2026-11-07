// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { MarkerColor } from "./MarkerColor";
import type { MarkerId } from "./MarkerId";
import type { SequenceId } from "./SequenceId";

/**
 * A marker and the sequence holding it.
 */
export type MarkerEntity = { 
/**
 * The sequence holding the marker.
 */
sequenceId: SequenceId, 
/**
 * Stable id.
 */
id: MarkerId, 
/**
 * Position in the sequence (≥ 0).
 */
time: Flicks, 
/**
 * Text shown next to the marker; may be empty.
 */
label: string, 
/**
 * Colour of the marker.
 */
color: MarkerColor, };
