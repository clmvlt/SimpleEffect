// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { Flicks } from "./Flicks";
import type { TimeRange } from "./TimeRange";
import type { TrackId } from "./TrackId";

/**
 * What a cut did (or, for a preview, would do).
 */
export type CutSummary = { 
/**
 * The ranges removed, in sequence time **before** the cut, sorted.
 */
cuts: Array<TimeRange>, 
/**
 * Their total duration (the time gained).
 */
removedDuration: Flicks, 
/**
 * Words removed (word and filler cuts; 0 for silences).
 */
wordCount: number, 
/**
 * The tracks cut (the speech's tracks and their linked partners, or every unlocked track).
 */
trackIds: Array<TrackId>, 
/**
 * Media of the range whose words are unknown (not transcribed yet): nothing was cut there.
 */
untranscribedAssets: Array<AssetId>, 
/**
 * Media of the range whose sound is not analysed yet: no silence was found there.
 */
unanalysedAssets: Array<AssetId>, };
