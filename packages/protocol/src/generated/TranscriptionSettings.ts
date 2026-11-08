// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * How the media of a project are transcribed (prompt 21). Changed by
 * [`crate::SetTranscriptionSettings`] (a user edit, undoable).
 */
export type TranscriptionSettings = { 
/**
 * The spoken language as an ISO 639 code (`fr`, `en`, `yue`…), or `None` to let the model
 * detect it for each media file.
 */
language: string | null, 
/**
 * Names and terms the model should spell this way (people, brands, jargon), as one line of
 * text given to it as context; empty for none.
 */
vocabulary: string, };
