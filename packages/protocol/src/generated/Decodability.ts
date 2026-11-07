// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { StreamDecodability } from "./StreamDecodability";

/**
 * Whether the renderer of this machine can decode each stream of the original, as it reported
 * after checking with WebCodecs (`isConfigSupported`, `docs/architecture.md` §7).
 */
export type Decodability = { 
/**
 * The picture stream (or the still picture).
 */
video: StreamDecodability, 
/**
 * The first sound stream.
 */
audio: StreamDecodability, };
