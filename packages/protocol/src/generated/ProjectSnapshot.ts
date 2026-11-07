// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Asset } from "./Asset";
import type { ItemEntity } from "./ItemEntity";
import type { MarkerEntity } from "./MarkerEntity";
import type { ProjectHeader } from "./ProjectHeader";
import type { SequenceEntity } from "./SequenceEntity";
import type { TrackEntity } from "./TrackEntity";

/**
 * Every entity of a project at one revision: what a client loads before applying patches.
 */
export type ProjectSnapshot = { 
/**
 * Revision of the document these entities describe; the next patch has `revision + 1`.
 */
revision: number, 
/**
 * The project's own fields.
 */
project: ProjectHeader, 
/**
 * Sequences, in project order.
 */
sequences: Array<SequenceEntity>, 
/**
 * Every track of every sequence.
 */
tracks: Array<TrackEntity>, 
/**
 * Every item of every track.
 */
items: Array<ItemEntity>, 
/**
 * Every marker of every sequence.
 */
markers: Array<MarkerEntity>, 
/**
 * Imported media, in import order.
 */
assets: Array<Asset>, };
