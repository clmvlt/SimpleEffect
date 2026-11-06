// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Format } from "./Format";
import type { Marker } from "./Marker";
import type { SequenceId } from "./SequenceId";
import type { Track } from "./Track";

/**
 * A timeline. A project may hold several (e.g. a 16:9 edit and its 9:16 version).
 */
export type Sequence = { 
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
 * Video and audio tracks; video tracks stack in this order (first = bottom).
 */
tracks: Array<Track>, 
/**
 * Labelled instants, in any order.
 */
markers: Array<Marker>, };
