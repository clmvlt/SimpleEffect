// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DerivedFile } from "./DerivedFile";

/**
 * The mezzanine and proxy of an asset. The original is [`super::Asset::original_path`].
 */
export type DerivedMedia = { 
/**
 * High-quality decodable copy, for originals WebCodecs cannot decode.
 */
mezzanine: DerivedFile | null, 
/**
 * Low-resolution copy for smooth playback.
 */
proxy: DerivedFile | null, };
