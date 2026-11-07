// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { SequenceId } from "./SequenceId";
import type { TrackId } from "./TrackId";
import type { TrackKind } from "./TrackKind";
import type { Transition } from "./Transition";

/**
 * A track header and the ids of its items.
 */
export type TrackEntity = { 
/**
 * Stable id.
 */
id: TrackId, 
/**
 * The sequence holding the track.
 */
sequenceId: SequenceId, 
/**
 * Video or audio.
 */
kind: TrackKind, 
/**
 * Name shown in the track header.
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
 * Items of this track cannot be edited.
 */
locked: boolean, 
/**
 * Pictures of this track are not drawn.
 */
hidden: boolean, 
/**
 * Track gain in decibels.
 */
volumeDb: number, 
/**
 * Stereo balance from −1 (left) to 1 (right).
 */
pan: number, 
/**
 * Its items, sorted by start time.
 */
itemIds: Array<ItemId>, 
/**
 * Its cross-fades (small values, sent whole with the track: a transition change upserts
 * its track).
 */
transitions: Array<Transition>, };
