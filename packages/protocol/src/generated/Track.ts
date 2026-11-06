// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Item } from "./Item";
import type { TrackId } from "./TrackId";
import type { TrackKind } from "./TrackKind";

/**
 * A horizontal lane of a sequence. Its items never overlap in time.
 *
 * Video tracks are composited in their order within [`super::Sequence::tracks`]: the first video
 * track is the bottom layer (V1), each following one is drawn above it.
 */
export type Track = { 
/**
 * Stable id, unique in the project.
 */
id: TrackId, 
/**
 * Video or audio; decides which items it may hold.
 */
kind: TrackKind, 
/**
 * Name shown in the track header (`V1`, `Voix off`…).
 */
name: string, 
/**
 * Audio of this track is silenced.
 */
muted: boolean, 
/**
 * Only solo tracks are heard when at least one track is solo.
 */
solo: boolean, 
/**
 * Items of this track cannot be edited (by the user or by agents).
 */
locked: boolean, 
/**
 * Pictures of this track are not drawn.
 */
hidden: boolean, 
/**
 * Track gain in decibels (0 = unchanged).
 */
volumeDb: number, 
/**
 * Stereo balance from −1 (left) to 1 (right).
 */
pan: number, 
/**
 * The items, in any order (operations keep them sorted by start).
 */
items: Array<Item>, };
