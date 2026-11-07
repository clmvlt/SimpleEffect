// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ClipReframePolicy } from "./ClipReframePolicy";

/**
 * How the items of a sequence adapt to its new format.
 */
export type FormatAdaptation = { 
/**
 * The reframe mode of video clips.
 */
clipReframe: ClipReframePolicy, 
/**
 * Gives texts wider than [`TEXT_WIDTH_SHARE`] of the new frame (wrapping width times their
 * horizontal scale; texts without wrapping count as too wide) that wrapping width, so long
 * lines wrap instead of leaving a narrower frame.
 */
fitTextWidths: boolean, };
