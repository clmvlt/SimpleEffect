// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { FaceBox } from "./FaceBox";
import type { Flicks } from "./Flicks";

/**
 * One sampled picture and the faces it shows.
 */
export type SubjectFrame = { 
/**
 * Source time of the picture (media time 0 is the first frame).
 */
time: Flicks, 
/**
 * The faces found, most confident first.
 */
faces: Array<FaceBox>, };
