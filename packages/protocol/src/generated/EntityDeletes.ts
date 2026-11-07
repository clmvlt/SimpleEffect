// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { ItemId } from "./ItemId";
import type { MarkerId } from "./MarkerId";
import type { SequenceId } from "./SequenceId";
import type { TrackId } from "./TrackId";

/**
 * Ids of the entities a commit removed.
 */
export type EntityDeletes = { 
/**
 * Sequences removed.
 */
sequences: Array<SequenceId>, 
/**
 * Tracks removed.
 */
tracks: Array<TrackId>, 
/**
 * Items removed.
 */
items: Array<ItemId>, 
/**
 * Markers removed.
 */
markers: Array<MarkerId>, 
/**
 * Assets removed.
 */
assets: Array<AssetId>, };
