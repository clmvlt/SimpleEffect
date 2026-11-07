// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { MarkerColor } from "./MarkerColor";

/**
 * Changes of a marker; `null` (or absent) fields are unchanged.
 */
export type MarkerPatch = { 
/**
 * Another position (≥ 0).
 */
time?: Flicks, 
/**
 * Another label (may be empty).
 */
label?: string, 
/**
 * Another colour.
 */
color?: MarkerColor, };
