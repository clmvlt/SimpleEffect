// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * The audio analysis of an asset as the document keeps it.
 */
export type AudioSummary = { 
/**
 * Integrated loudness in LUFS (`None`: silent).
 */
integratedLufs: number | null, 
/**
 * Loudness range in LU.
 */
loudnessRangeLu: number | null, 
/**
 * Highest true peak in dBTP.
 */
truePeakDbtp: number | null, 
/**
 * Number of silences detected.
 */
silenceCount: number, 
/**
 * Their total duration.
 */
silenceDuration: Flicks, };
