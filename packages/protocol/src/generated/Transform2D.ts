// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Vec2 } from "./Vec2";

/**
 * Placement of a visual item, every property animatable. Units: ADR-0015 (`crate::spatial`).
 */
export type Transform2D = { 
/**
 * Where the anchor sits, normalised in the sequence frame (`(0.5, 0.5)` is the centre).
 */
position: Animatable<Vec2>, 
/**
 * Relative size (`(1, 1)` is the natural size; negative mirrors).
 */
scale: Animatable<Vec2>, 
/**
 * Clockwise rotation around the anchor, in degrees.
 */
rotation: Animatable<number>, 
/**
 * The item's pivot, normalised in its own box (`(0.5, 0.5)` is its centre).
 */
anchor: Animatable<Vec2>, 
/**
 * Opacity from 0 (invisible) to 1 (opaque).
 */
opacity: Animatable<number>, };
