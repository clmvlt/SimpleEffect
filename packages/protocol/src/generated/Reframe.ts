// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { TrackingSettings } from "./TrackingSettings";
import type { Vec2 } from "./Vec2";

/**
 * How a clip's picture fits the sequence frame before its transform applies (ADR-0026).
 *
 * The fill modes crop the picture to the frame's aspect around a **focus**: a point of the
 * upright picture, normalised in it (`(0, 0)` top-left, `(1, 1)` bottom-right), kept at the
 * frame's centre as far as the picture allows (the crop never leaves the picture). The focus
 * does not depend on the frame's aspect, so it survives format changes and versions in other
 * formats; it is the animatable property [`crate::AnimatableProperty::ReframeFocus`].
 */
export type Reframe = { "kind": "fit" } | { "kind": "fill", 
/**
 * The point of the picture kept at the frame's centre (centre when absent, as in
 * documents written before schema version 6).
 */
focus: Animatable<Vec2>, } | { "kind": "custom" } | { "kind": "tracked", 
/**
 * The point of the picture kept at the frame's centre over time.
 */
focus: Animatable<Vec2>, 
/**
 * How the trajectory was computed (schema version 11), reused when it is recomputed.
 */
tracking: TrackingSettings, };
