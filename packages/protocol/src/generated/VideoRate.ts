// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ExportQuality } from "./ExportQuality";

/**
 * How the picture's bitrate is chosen (ignored by ProRes and DNxHR, whose profiles fix it).
 */
export type VideoRate = { "kind": "bitrate", 
/**
 * Bits per second.
 */
bitsPerSecond: number, } | { "kind": "quality", 
/**
 * The level.
 */
level: ExportQuality, };
