// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";

/**
 * Colours and fonts of the user's brand.
 *
 * TODO(prompt-25): logo asset, default caption style, named colour roles and the global library.
 */
export type BrandKit = { 
/**
 * Brand colours, most important first.
 */
colors: Array<Color>, 
/**
 * Font family names, most important first.
 */
fonts: Array<string>, };
