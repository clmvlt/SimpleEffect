// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * One spoken word in source time.
 */
export type TranscriptWord = { 
/**
 * The word as written, with the punctuation attached to it (`va ?`, `l'homme,`).
 */
text: string, 
/**
 * When it starts in the asset.
 */
start: Flicks, 
/**
 * When it ends in the asset.
 */
end: Flicks, 
/**
 * How sure the model is, from 0 to 1 (mean probability of its tokens).
 */
confidence: number, 
/**
 * Index of its segment in [`Transcript::segments`].
 */
segment: number, };
