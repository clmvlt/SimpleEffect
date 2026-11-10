// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { VideoExportCodec } from "./VideoExportCodec";
import type { VideoRate } from "./VideoRate";

/**
 * Picture settings of a saved preset: the output keeps the sequence's aspect, the preset fixes
 * its shorter side.
 */
export type ExportPresetVideo = { 
/**
 * Its codec.
 */
codec: VideoExportCodec, 
/**
 * The shorter side in pixels (1080 for « 1080p »).
 */
shortSide: number, 
/**
 * Its bitrate.
 */
rate: VideoRate, };
