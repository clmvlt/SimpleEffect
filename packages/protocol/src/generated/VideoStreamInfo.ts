// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { FrameRate } from "./FrameRate";

/**
 * The main picture stream of a file.
 */
export type VideoStreamInfo = { 
/**
 * Codec name (e.g. `h264`, `hevc`, `prores`).
 */
codec: string, 
/**
 * Codec profile (e.g. `High`, `Main 10`), when known.
 */
profile: string | null, 
/**
 * Bits per colour component (8, 10, 12), when known.
 */
bitDepth: number | null, 
/**
 * Chroma subsampling (`4:2:0`, `4:2:2`, `4:4:4`), when known.
 */
chromaSubsampling: string | null, 
/**
 * Coded width in pixels, before rotation.
 */
width: number, 
/**
 * Coded height in pixels, before rotation.
 */
height: number, 
/**
 * Nominal frame rate; `None` if unknown or not exactly representable in flicks (variable
 * frame rate footage is conformed through a mezzanine).
 */
frameRate: FrameRate | null, 
/**
 * Clockwise display rotation in degrees: 0, 90, 180 or 270 (phone footage).
 */
rotation: number, };
