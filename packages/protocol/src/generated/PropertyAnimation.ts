// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Vec2 } from "./Vec2";

/**
 * An animatable property with its new value, typed by property (positions are [`Vec2`],
 * rotations numbers…). Serialized as `{"property": "opacity", "value": {"kind": "keyframed", …}}`.
 */
export type PropertyAnimation = { "property": "position", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "scale", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "rotation", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, } | { "property": "anchor", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "opacity", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, } | { "property": "volumeDb", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, };
