// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { VideoExportCodec } from "./VideoExportCodec";
import type { VideoRate } from "./VideoRate";

/**
 * The picture of an export.
 */
export type VideoExportSettings = { 
/**
 * Its codec.
 */
codec: VideoExportCodec, 
/**
 * Width in pixels (even); the sequence frame is scaled to it.
 */
width: number, 
/**
 * Height in pixels (even).
 */
height: number, 
/**
 * Its bitrate.
 */
rate: VideoRate, };
