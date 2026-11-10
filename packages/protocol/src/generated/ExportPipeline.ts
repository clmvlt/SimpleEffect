// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * How the file of an export is put together (decided by the core from the settings).
 */
export type ExportPipeline = { "kind": "muxed" } | { "kind": "ffmpegFrames", 
/**
 * Whether a WAV comes first.
 */
withAudio: boolean, } | { "kind": "aacRemux", 
/**
 * Whether a picture file is written too.
 */
withVideo: boolean, };
