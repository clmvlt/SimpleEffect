// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Color } from "./Color";
import type { ShapeGeometry } from "./ShapeGeometry";
import type { Stroke } from "./Stroke";
import type { Transform2D } from "./Transform2D";

/**
 * A filled vector shape with an optional outline.
 *
 * Dimensions are static: animate `scale` to grow a shape (animatable dimensions and gradients
 * are in `docs/backlog.md`).
 */
export type ShapeItem = { 
/**
 * The outline.
 */
geometry: ShapeGeometry, 
/**
 * Fill colour.
 */
fill: Animatable<Color>, 
/**
 * Outline drawn along the shape's edge (outside it).
 */
stroke: Stroke, 
/**
 * The box is the whole sequence frame, whatever its format (a backdrop, a colour plate):
 * the geometry's width and height are ignored (ADR-0015). Not used by lines.
 */
coverFrame: boolean, 
/**
 * Placement of the shape's box.
 */
transform: Transform2D, };
