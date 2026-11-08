// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * What an asset keeps of its transcript (in the document): enough to show a badge and to know
 * which transcript version the renderer holds.
 */
export type TranscriptSummary = { 
/**
 * The spoken language.
 */
language: string, 
/**
 * The model that transcribed it.
 */
model: string, 
/**
 * How many words were recognised.
 */
wordCount: number, 
/**
 * When it was transcribed (Unix milliseconds): a new transcription replaces the old one and
 * changes this value.
 */
transcribedAt: number, };
