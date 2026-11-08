// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { Vec2 } from "./Vec2";

/**
 * A drop shadow of caption glyphs.
 */
export type CaptionShadow = { 
/**
 * Whether it is drawn.
 */
enabled: boolean, 
/**
 * Shadow colour.
 */
color: Color, 
/**
 * Blur radius in design pixels; ≥ 0.
 */
blur: DesignPixels, 
/**
 * Offset in design pixels (rightward, downward).
 */
offset: Vec2, };
