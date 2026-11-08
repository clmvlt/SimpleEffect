// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * How silences are found and cut.
 */
export type SilenceCutParams = { 
/**
 * Level (dBFS, RMS of 20 ms windows) under which sound counts as silence (−80 to −10).
 */
thresholdDbfs: number, 
/**
 * Shortest silence cut (0.1 s to 10 s).
 */
minSilence: Flicks, 
/**
 * Silence kept before and after each cut so speech breathes (0 to 1 s).
 */
padding: Flicks, };
