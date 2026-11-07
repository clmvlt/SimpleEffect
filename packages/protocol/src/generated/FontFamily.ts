// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { FontFace } from "./FontFace";
import type { FontSource } from "./FontSource";

/**
 * A font family and its faces.
 */
export type FontFamily = { 
/**
 * Family name, as texts name it (`fontFamily`).
 */
name: string, 
/**
 * Where it comes from.
 */
source: FontSource, 
/**
 * Its faces (weights and styles), lightest first.
 */
faces: Array<FontFace>, };
