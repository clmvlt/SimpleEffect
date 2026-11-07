// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AnimationPhase } from "./AnimationPhase";
import type { PropertyAnimation } from "./PropertyAnimation";

/**
 * What a preset does in one phase to the reference text of [`AnimationPresetInfo::samples`].
 */
export type AnimationPresetSample = { 
/**
 * The phase.
 */
phase: AnimationPhase, 
/**
 * The animated properties (keyframes over one second).
 */
animations: Array<PropertyAnimation>, };
