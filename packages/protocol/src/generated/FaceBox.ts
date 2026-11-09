// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * A face found in a picture: its box normalised in the upright picture (`(0, 0)` top-left,
 * `(1, 1)` bottom-right, like a reframe focus) and the detector's confidence.
 */
export type FaceBox = { 
/**
 * Left edge.
 */
x: number, 
/**
 * Top edge.
 */
y: number, 
/**
 * Width.
 */
width: number, 
/**
 * Height.
 */
height: number, 
/**
 * Confidence in `[0, 1]`.
 */
score: number, };
