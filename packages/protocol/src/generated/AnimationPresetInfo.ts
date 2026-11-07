// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AnimatableProperty } from "./AnimatableProperty";
import type { AnimationPhase } from "./AnimationPhase";
import type { AnimationPresetSample } from "./AnimationPresetSample";
import type { Flicks } from "./Flicks";

/**
 * An animation preset of the library (`listAnimationPresets`), with what it does to a
 * reference text so clients can preview it without expanding it themselves.
 */
export type AnimationPresetInfo = { 
/**
 * Stable id, given to `applyAnimationPreset`.
 */
id: string, 
/**
 * English name.
 */
name: string, 
/**
 * What it does.
 */
description: string, 
/**
 * Phases it can be applied as (`in` and `out`, or `emphasis`).
 */
phases: Array<AnimationPhase>, 
/**
 * Suggested duration.
 */
defaultDuration: Flicks, 
/**
 * Properties it animates (an item must have them all).
 */
properties: Array<AnimatableProperty>, 
/**
 * Its expansion on a centred « Aa » text lasting one second, per phase.
 */
samples: Array<AnimationPresetSample>, };
