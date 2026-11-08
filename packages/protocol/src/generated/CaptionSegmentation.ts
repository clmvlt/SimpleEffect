// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * Overrides of the segmentation limits (`null` fields keep the style's line limits and the
 * defaults: 0.7 s to 6 s, a new caption after a 0.6 s pause).
 */
export type CaptionSegmentation = { 
/**
 * Longest line in characters (4–80).
 */
maxCharsPerLine?: number, 
/**
 * Lines per caption (1–4).
 */
maxLines?: number, 
/**
 * Shortest caption.
 */
minDuration?: Flicks, 
/**
 * Longest caption.
 */
maxDuration?: Flicks, 
/**
 * A pause at least this long starts a new caption.
 */
pause?: Flicks, };
