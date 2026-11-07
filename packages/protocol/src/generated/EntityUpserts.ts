// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Asset } from "./Asset";
import type { ItemEntity } from "./ItemEntity";
import type { MarkerEntity } from "./MarkerEntity";
import type { ProjectHeader } from "./ProjectHeader";
import type { SequenceEntity } from "./SequenceEntity";
import type { TrackEntity } from "./TrackEntity";

/**
 * Entities added or changed by a commit.
 */
export type EntityUpserts = { 
/**
 * The project's own fields, when they changed.
 */
project: ProjectHeader | null, 
/**
 * Sequences added or changed (including their track and marker lists).
 */
sequences: Array<SequenceEntity>, 
/**
 * Tracks added or changed (including their item lists).
 */
tracks: Array<TrackEntity>, 
/**
 * Items added or changed.
 */
items: Array<ItemEntity>, 
/**
 * Markers added or changed.
 */
markers: Array<MarkerEntity>, 
/**
 * Assets added or changed.
 */
assets: Array<Asset>, };
