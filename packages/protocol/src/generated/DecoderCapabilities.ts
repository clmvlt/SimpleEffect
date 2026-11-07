// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * What this machine's WebCodecs decodes, reported by the renderer (`isConfigSupported`) once per
 * connection. Decides the codecs of new mezzanines and proxies.
 */
export type DecoderCapabilities = { 
/**
 * HEVC Main 10 (4:2:0, 10 bits) up to 4K: mezzanines of 10-bit footage keep their depth.
 */
hevcMain10: boolean, 
/**
 * H.264 High (4:2:0, 8 bits) up to 4K: the default mezzanine and proxy codec.
 */
h264High: boolean, 
/**
 * AAC-LC: the sound of mezzanines and proxies; FLAC otherwise.
 */
aac: boolean, 
/**
 * FLAC.
 */
flac: boolean, };
