// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Whether WebCodecs can decode and encode one codec here.
 */
export type CodecSupport = { 
/**
 * The codec (`H.264`, `HEVC`, `VP9`, `AV1`).
 */
codec: string, 
/**
 * Decoding is supported.
 */
decode: boolean, 
/**
 * Encoding is supported.
 */
encode: boolean, };
