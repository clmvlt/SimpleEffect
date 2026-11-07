// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * One sound stream of a file.
 */
export type AudioStreamInfo = { 
/**
 * Codec name (e.g. `aac`, `pcm_s24le`).
 */
codec: string, 
/**
 * Codec profile (e.g. `LC`, `HE-AAC`), when known. Needed to build WebCodecs codec strings.
 */
profile: string | null, 
/**
 * Number of channels (1 mono, 2 stereo…).
 */
channels: number, 
/**
 * Samples per second (e.g. 48000).
 */
sampleRate: number, };
