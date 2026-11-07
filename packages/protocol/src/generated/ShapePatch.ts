// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { ShapeGeometry } from "./ShapeGeometry";

/**
 * Changes of a [`crate::ShapeItem`].
 */
export type ShapePatch = { 
/**
 * Another outline.
 */
geometry: ShapeGeometry | null, 
/**
 * Another fill colour.
 */
fill: Color | null, };
