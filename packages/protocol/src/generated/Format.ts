// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { FrameRate } from "./FrameRate";

/**
 * Frame size, frame rate and audio sample rate of a sequence.
 *
 * Validation requires even dimensions between 2 and 16384 (video encoders need them) and a
 * sample rate whose sample lasts a whole number of flicks (44.1, 48, 96 kHz…).
 */
export type Format = { 
/**
 * Frame width in pixels.
 */
width: number, 
/**
 * Frame height in pixels.
 */
height: number, 
/**
 * Frames per second.
 */
frameRate: FrameRate, 
/**
 * Audio samples per second.
 */
sampleRate: number, };
