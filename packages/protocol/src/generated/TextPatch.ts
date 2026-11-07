// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";

/**
 * Changes of a [`crate::TextItem`].
 *
 * TODO(prompt-15): the style fields added to `TextStyle`.
 */
export type TextPatch = { 
/**
 * Another text.
 */
content: string | null, 
/**
 * Another font family.
 */
fontFamily: string | null, 
/**
 * Another font size.
 */
fontSize: DesignPixels | null, 
/**
 * Another glyph colour.
 */
color: Color | null, };
