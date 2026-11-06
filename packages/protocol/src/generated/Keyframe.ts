// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Easing } from "./Easing";
import type { Flicks } from "./Flicks";

/**
 * The value of a property at an instant, and how to reach the next keyframe.
 */
export type Keyframe<T> = { 
/**
 * Time relative to the item's start (`0` is its first instant, its duration its end).
 */
time: Flicks, 
/**
 * The value at that time.
 */
value: T, 
/**
 * Interpolation from this keyframe to the next one (ignored on the last keyframe).
 */
easing: Easing, };
