// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { Vec2 } from "./Vec2";

/**
 * A plate behind caption text.
 */
export type CaptionBox = { 
/**
 * Whether it is drawn.
 */
enabled: boolean, 
/**
 * Plate colour.
 */
color: Color, 
/**
 * Space around the text in design pixels (x left and right, y above and below); ≥ 0.
 */
padding: Vec2, 
/**
 * Corner radius in design pixels; ≥ 0.
 */
cornerRadius: DesignPixels, };
