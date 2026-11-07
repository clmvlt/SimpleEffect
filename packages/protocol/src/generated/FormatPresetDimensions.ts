// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { FormatPreset } from "./FormatPreset";

/**
 * A named format preset and its frame size.
 */
export type FormatPresetDimensions = { 
/**
 * The preset.
 */
preset: FormatPreset, 
/**
 * Frame width in pixels.
 */
width: number, 
/**
 * Frame height in pixels.
 */
height: number, };
