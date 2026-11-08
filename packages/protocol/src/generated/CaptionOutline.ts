// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";

/**
 * An outline around caption glyphs, all of it outside the glyphs.
 */
export type CaptionOutline = { 
/**
 * Whether it is drawn.
 */
enabled: boolean, 
/**
 * Outline colour.
 */
color: Color, 
/**
 * Visible width in design pixels; ≥ 0.
 */
width: DesignPixels, };
