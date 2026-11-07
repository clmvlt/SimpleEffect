// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { Vec2 } from "./Vec2";

/**
 * A plate behind the text block (lower thirds, captions). Kept when disabled.
 */
export type TextBackground = { 
/**
 * Whether the plate is drawn. When drawn, the item's box includes it.
 */
enabled: boolean, 
/**
 * Plate colour.
 */
color: Animatable<Color>, 
/**
 * Space between the text and the plate's edges, in design pixels (x on the left and right,
 * y above and below); ≥ 0.
 */
padding: Vec2, 
/**
 * Radius of the plate's corners in design pixels; ≥ 0.
 */
cornerRadius: DesignPixels, };
