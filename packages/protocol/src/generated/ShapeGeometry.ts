// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DesignPixels } from "./DesignPixels";

/**
 * The outline of a shape, sized in design pixels (ADR-0015).
 */
export type ShapeGeometry = { "kind": "rectangle", 
/**
 * Width of the box.
 */
width: DesignPixels, 
/**
 * Height of the box.
 */
height: DesignPixels, 
/**
 * Radius of the rounded corners (0 for square corners).
 */
cornerRadius: DesignPixels, } | { "kind": "ellipse", 
/**
 * Width of the box.
 */
width: DesignPixels, 
/**
 * Height of the box.
 */
height: DesignPixels, };
