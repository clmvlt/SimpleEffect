// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionStyleId } from "./CaptionStyleId";

/**
 * Which style new captions get.
 */
export type CaptionStyleChoice = { "kind": "preset", 
/**
 * The preset's id.
 */
preset: string, } | { "kind": "existing", 
/**
 * Its id.
 */
styleId: CaptionStyleId, } | { "kind": "brandKit" };
