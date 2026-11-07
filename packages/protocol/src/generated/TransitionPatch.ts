// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { TransitionAlignment } from "./TransitionAlignment";

/**
 * Changes of a [`Transition`]: its length and alignment. Its kind follows its track and its
 * items are fixed (remove it and add another to join other items).
 */
export type TransitionPatch = { 
/**
 * Another duration (> 0).
 */
duration?: Flicks, 
/**
 * Another alignment on the cut.
 */
alignment?: TransitionAlignment, };
