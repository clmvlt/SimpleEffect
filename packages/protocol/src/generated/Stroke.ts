// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";

/**
 * An outline. Kept when disabled so turning it back on restores it.
 */
export type Stroke = { 
/**
 * Whether the outline is drawn.
 */
enabled: boolean, 
/**
 * Outline colour.
 */
color: Animatable<Color>, 
/**
 * Visible width of the outline outside the glyphs or the shape's edge, in design pixels;
 * ≥ 0.
 */
width: Animatable<DesignPixels>, };
