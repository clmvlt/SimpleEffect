// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";

/**
 * Minimal text style.
 *
 * TODO(prompt-15): weight, alignment, line height, stroke, shadow, background and the text
 * animations.
 */
export type TextStyle = { 
/**
 * Font family name (a bundled or brand-kit font).
 */
fontFamily: string, 
/**
 * Font size in design pixels (ADR-0015).
 */
fontSize: DesignPixels, 
/**
 * Fill colour of the glyphs.
 */
color: Color, };
