// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { TextStyle } from "./TextStyle";
import type { Transform2D } from "./Transform2D";

/**
 * A block of text. Editable data, never a rendered picture (golden rule 6).
 */
export type TextItem = { 
/**
 * The text, lines separated by `\n`.
 */
content: string, 
/**
 * How the text looks.
 */
style: TextStyle, 
/**
 * Placement of the text box.
 */
transform: Transform2D, 
/**
 * Share of the characters shown, from 0 (none) to 1 (all), in reading order: animating it
 * from 0 to 1 types the text out (the « machine à écrire » preset). The layout is always
 * that of the whole text, so revealed characters never move.
 */
reveal: Animatable<number>, };
