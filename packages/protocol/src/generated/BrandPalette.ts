// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Color } from "./Color";

/**
 * Colours by role.
 */
export type BrandPalette = { 
/**
 * Main brand colour.
 */
primary: Color, 
/**
 * Second brand colour.
 */
secondary: Color, 
/**
 * Highlights, calls to action.
 */
accent: Color, 
/**
 * Text over the background colour.
 */
text: Color, 
/**
 * Plates and backgrounds.
 */
background: Color, };
