// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { Vec2 } from "./Vec2";

/**
 * A drop shadow. Kept when disabled so turning it back on restores it.
 */
export type TextShadow = { 
/**
 * Whether the shadow is drawn.
 */
enabled: boolean, 
/**
 * Shadow colour (its alpha is the shadow's opacity).
 */
color: Animatable<Color>, 
/**
 * Blur radius in design pixels; ≥ 0.
 */
blur: Animatable<DesignPixels>, 
/**
 * Shift of the shadow in design pixels (x rightward, y downward).
 */
offset: Animatable<Vec2>, };
