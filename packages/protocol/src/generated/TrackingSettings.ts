// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The settings of a tracked reframe. Every value has a range ([`TrackingSettings::problems`]);
 * the trajectory itself is computed by [`crate::tracking`].
 */
export type TrackingSettings = { 
/**
 * How smooth the frame's movement is, from 0 (follows the subject closely) to 1 (slow,
 * steady moves).
 */
smoothing: number, 
/**
 * Where the subject's eyes sit in the frame, measured from its top in `[0.2, 0.8]` (0.33:
 * the upper third line). It only matters when the frame keeps less than the picture's
 * height (a portrait picture in a landscape frame, a zoom).
 */
eyeLine: number, 
/**
 * Frame every face when they fit in the frame together; otherwise follow the main subject.
 */
groupFraming: boolean, 
/**
 * Share of the frame, in `[0, 0.5]`, the subject may move across before the frame follows
 * it (a camera operator's « dead zone »; 0 follows every move).
 */
deadZone: number, };
