// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * One face of a family: a file (or part of one) covering some weights and a style.
 */
export type FontFace = { 
/**
 * Id of the face, for `GET /media/fonts/{id}`.
 */
id: string, 
/**
 * Lightest weight it draws (100 to 900; variable fonts cover a range).
 */
weightMin: number, 
/**
 * Heaviest weight it draws.
 */
weightMax: number, 
/**
 * Italic or oblique.
 */
italic: boolean, 
/**
 * The CSS `unicode-range` it covers (`U+0000-00FF, U+0131`), when it is a subset.
 */
unicodeRange: string | null, 
/**
 * File format for CSS (`woff2`, `truetype`, `opentype`, `collection`).
 */
format: string, };
