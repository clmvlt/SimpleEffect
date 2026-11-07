// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { ItemId } from "./ItemId";
import type { TrackId } from "./TrackId";
import type { TransitionAlignment } from "./TransitionAlignment";

/**
 * A cross-fade wanted on a cut (`addTransitions`): its kind follows the track (dissolve on video
 * tracks, sound cross-fade on audio tracks).
 */
export type NewTransition = { 
/**
 * The track of the cut.
 */
trackId: TrackId, 
/**
 * The item ending at the cut.
 */
leftItemId: ItemId, 
/**
 * The item starting at the cut.
 */
rightItemId: ItemId, 
/**
 * The wanted length (shortened to what the cut allows).
 */
duration: Flicks, 
/**
 * Where the transition sits on the cut.
 */
alignment: TransitionAlignment, };
