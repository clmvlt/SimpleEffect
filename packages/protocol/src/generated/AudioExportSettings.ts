// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AudioEncoderChoice } from "./AudioEncoderChoice";
import type { AudioExportCodec } from "./AudioExportCodec";

/**
 * The sound of an export.
 */
export type AudioExportSettings = { 
/**
 * Its codec.
 */
codec: AudioExportCodec, 
/**
 * Bits per second (ignored for PCM).
 */
bitsPerSecond: number, 
/**
 * Who encodes it.
 */
encoder: AudioEncoderChoice, };
