// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionStyleId } from "./CaptionStyleId";
import type { CaptionWord } from "./CaptionWord";

/**
 * A subtitle: timed words drawn with a project caption style.
 *
 * Word times are relative to the item's start (like keyframes) and lie inside the item; words
 * are listed in speaking order and never overlap (validation: `INVALID_CAPTION_WORDS`). A caption
 * may hold no word (a trim left none): it then draws nothing.
 */
export type CaptionItem = { 
/**
 * The words, in speaking order.
 */
words: Array<CaptionWord>, 
/**
 * The project caption style drawing it ([`crate::Project::caption_styles`]).
 */
styleId: CaptionStyleId, };
