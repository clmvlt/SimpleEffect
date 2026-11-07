// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Changes of a track header; `null` (or absent) fields are unchanged. The kind of a track
 * cannot change (its items depend on it).
 */
export type TrackPatch = { 
/**
 * Another name (not empty).
 */
name?: string, 
/**
 * Mute or unmute.
 */
muted?: boolean, 
/**
 * Solo or unsolo.
 */
solo?: boolean, 
/**
 * Lock or unlock.
 */
locked?: boolean, 
/**
 * Hide or show.
 */
hidden?: boolean, 
/**
 * Another gain in decibels.
 */
volumeDb?: number, 
/**
 * Another stereo balance (−1 to 1).
 */
pan?: number, };
