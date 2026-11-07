// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { ShapeGeometry } from "./ShapeGeometry";

/**
 * Changes of a [`crate::ShapeItem`] (its animatable colours and outline width go through
 * [`super::SetKeyframes`]; `fill` here sets a static value).
 */
export type ShapePatch = { 
/**
 * Another outline.
 */
geometry?: ShapeGeometry, 
/**
 * A static fill colour (replaces keyframes).
 */
fill?: Color, 
/**
 * Outline on or off.
 */
strokeEnabled?: boolean, 
/**
 * Whether the box covers the whole frame.
 */
coverFrame?: boolean, };
