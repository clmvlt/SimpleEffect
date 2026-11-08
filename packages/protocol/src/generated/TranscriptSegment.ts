// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * A sentence or phrase of a transcript.
 */
export type TranscriptSegment = { 
/**
 * When it starts in the asset.
 */
start: Flicks, 
/**
 * When it ends in the asset.
 */
end: Flicks, };
