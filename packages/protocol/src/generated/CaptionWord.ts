// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * One word of a caption: its text and when it is spoken.
 */
export type CaptionWord = { 
/**
 * The word as shown (punctuation attached), never empty, without line breaks.
 */
text: string, 
/**
 * When it starts being spoken, relative to the item's start.
 */
start: Flicks, 
/**
 * When it stops (≥ `start`, ≤ the next word's start), relative to the item's start.
 */
end: Flicks, 
/**
 * Shown in the style's emphasis colour (a keyword put forward).
 */
emphasis: boolean, };
