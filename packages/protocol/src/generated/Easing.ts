// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { EasingPreset } from "./EasingPreset";

/**
 * Interpolation curve from one keyframe to the next.
 */
export type Easing = { "kind": "linear" } | { "kind": "hold" } | { "kind": "cubicBezier", 
/**
 * Time of the first control point.
 */
x1: number, 
/**
 * Progress of the first control point.
 */
y1: number, 
/**
 * Time of the second control point.
 */
x2: number, 
/**
 * Progress of the second control point.
 */
y2: number, } | { "kind": "preset", 
/**
 * Which preset.
 */
name: EasingPreset, };
