// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TranscriptSegment } from "./TranscriptSegment";
import type { TranscriptWord } from "./TranscriptWord";

/**
 * The words spoken in an asset, in source time.
 *
 * Invariants (guaranteed by the transcriber adapter, checked by [`Transcript::problems`]): words
 * are sorted by start, never overlap (`end ≤` the next word's `start`), `start ≤ end`,
 * confidences lie in `[0, 1]` and segment indices point into `segments`.
 */
export type Transcript = { 
/**
 * The spoken language (ISO 639 code), forced or detected.
 */
language: string, 
/**
 * The model that transcribed it (`large-v3-turbo`, or a custom model's file name).
 */
model: string, 
/**
 * Every word, in order.
 */
words: Array<TranscriptWord>, 
/**
 * Sentences or phrases as the model cut them.
 */
segments: Array<TranscriptSegment>, };
