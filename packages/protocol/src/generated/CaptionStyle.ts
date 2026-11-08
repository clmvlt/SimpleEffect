// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionAnimation } from "./CaptionAnimation";
import type { CaptionBox } from "./CaptionBox";
import type { CaptionOutline } from "./CaptionOutline";
import type { CaptionShadow } from "./CaptionShadow";
import type { CaptionStyleId } from "./CaptionStyleId";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { TextCase } from "./TextCase";
import type { Vec2 } from "./Vec2";

/**
 * A caption look shared by captions of a project (`CaptionItem::style_id`).
 *
 * Sizes are design pixels (ADR-0015); `position` is the bottom centre of the caption block,
 * normalised in the frame, so blocks of one or more lines keep their baseline area.
 */
export type CaptionStyle = { 
/**
 * Stable id.
 */
id: CaptionStyleId, 
/**
 * The predefined style it was made from (`tiktokBold`, `minimal`…), kept to name it in the
 * interface; `null` for styles made from scratch.
 */
preset: string | null, 
/**
 * Font family (catalogue name, e.g. `Montserrat`).
 */
fontFamily: string, 
/**
 * Font weight, 1–1000.
 */
fontWeight: number, 
/**
 * Italic face.
 */
italic: boolean, 
/**
 * Font size in design pixels; > 0.
 */
fontSize: DesignPixels, 
/**
 * Letter case of the shown text.
 */
textCase: TextCase, 
/**
 * Distance between baselines as a multiple of the font size, in `(0, 10]`.
 */
lineHeight: number, 
/**
 * Colour of words not spoken yet (and of every word without animation).
 */
textColor: Color, 
/**
 * Colour of the word being spoken (and, with `highlight_spoken`, of the words spoken).
 */
activeColor: Color, 
/**
 * Colour of emphasised words (`CaptionWord::emphasis`) when they are not active.
 */
emphasisColor: Color, 
/**
 * Outline around the glyphs.
 */
outline: CaptionOutline, 
/**
 * Drop shadow of the glyphs.
 */
shadow: CaptionShadow, 
/**
 * Plate behind the block (behind each line with [`CaptionAnimation::BoxPerLine`]).
 */
background: CaptionBox, 
/**
 * Bottom centre of the block, normalised in the frame (both components in `[0, 1]`).
 */
position: Vec2, 
/**
 * How words animate as they are spoken.
 */
animation: CaptionAnimation, 
/**
 * Words already spoken keep the active colour (karaoke fill).
 */
highlightSpoken: boolean, 
/**
 * Longest line in characters ([`CAPTION_LINE_CHARS`]); longer words get a line of their own.
 */
maxCharsPerLine: number, 
/**
 * Lines shown at once ([`CAPTION_LINES`]); a longer caption shows the lines holding the
 * active word.
 */
maxLines: number, };
