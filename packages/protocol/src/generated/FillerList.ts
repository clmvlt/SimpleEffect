// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The filler words of one language.
 */
export type FillerList = { 
/**
 * ISO 639-1 code of the language (`fr`, `en`).
 */
language: string, 
/**
 * Words or short phrases (`euh`, `you know`), matched case-, accent- and
 * punctuation-insensitively; repeated letters match once (`euuuh` is `euh`).
 */
words: Array<string>, };
