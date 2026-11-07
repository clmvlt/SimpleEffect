// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { ItemId } from "./ItemId";
import type { TransitionAlignment } from "./TransitionAlignment";
import type { TransitionId } from "./TransitionId";
import type { TransitionKind } from "./TransitionKind";

/**
 * A cross-fade over the cut between two adjacent items of the same track: during its span the
 * outgoing item keeps playing past its end and the incoming one starts before its start, using
 * their **handles** (media beyond their edges, see [`super::item_handles`]).
 *
 * The track lists its transitions; the items do not refer to them. Rules (checked by
 * [`crate::validate_project`]): both items are on the track, `left_item` ends where
 * `right_item` starts, the kind matches the track kind, the span fits in both items and their
 * handles, and an item takes part in at most one transition on each side, without overlap.
 */
export type Transition = { 
/**
 * Stable id, unique in the project.
 */
id: TransitionId, 
/**
 * Picture dissolve or sound cross-fade.
 */
kind: TransitionKind, 
/**
 * The outgoing item (it ends at the cut).
 */
leftItem: ItemId, 
/**
 * The incoming item (it starts at the cut).
 */
rightItem: ItemId, 
/**
 * Length of the cross-fade (> 0).
 */
duration: Flicks, 
/**
 * Where the span sits relative to the cut.
 */
alignment: TransitionAlignment, };
