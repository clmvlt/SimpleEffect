// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";
import type { ShapeGeometry } from "./ShapeGeometry";
import type { Transform2D } from "./Transform2D";

/**
 * A filled vector shape.
 *
 * TODO(prompt-15): stroke, gradients, more geometries and animatable dimensions.
 */
export type ShapeItem = { 
/**
 * The outline.
 */
geometry: ShapeGeometry, 
/**
 * Fill colour.
 */
fill: Color, 
/**
 * Placement of the shape's box.
 */
transform: Transform2D, };
