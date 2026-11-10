// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * What ffprobe found in a finished export.
 */
export type ExportCheck = { 
/**
 * Duration of the file, in seconds.
 */
durationSeconds: number, 
/**
 * Codec of its picture (`h264`, `prores`…), if it has one.
 */
videoCodec: string | null, 
/**
 * Size of its picture, if it has one.
 */
width: number | null, 
/**
 * Height of its picture.
 */
height: number | null, 
/**
 * Codec of its sound (`aac`, `opus`, `pcm_s16le`…), if it has one.
 */
audioCodec: string | null, 
/**
 * What looks wrong (a duration or stream that differs from what was asked), if anything.
 */
warning: string | null, };
