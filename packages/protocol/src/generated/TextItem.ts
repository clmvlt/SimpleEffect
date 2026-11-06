// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
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
transform: Transform2D, };
