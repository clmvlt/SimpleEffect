// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionStyleId } from "./CaptionStyleId";
import type { CaptionWord } from "./CaptionWord";

/**
 * Changes of a [`crate::CaptionItem`] (clients may omit unchanged fields).
 */
export type CaptionPatch = { 
/**
 * Other words (text, timing, emphasis), item-relative and inside the item.
 */
words?: Array<CaptionWord>, 
/**
 * Another project caption style.
 */
styleId?: CaptionStyleId, };
