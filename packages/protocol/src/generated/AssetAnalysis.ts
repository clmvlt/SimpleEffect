// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AudioSummary } from "./AudioSummary";
import type { SceneSummary } from "./SceneSummary";

/**
 * The analysis summaries of an asset, kept in the document. `None` until the analysis ran on
 * the asset's current content (relinking to another content clears them).
 */
export type AssetAnalysis = { 
/**
 * Loudness, peaks and silences of the first sound stream.
 */
audio: AudioSummary | null, 
/**
 * Shot changes of the picture.
 */
scenes: SceneSummary | null, };
