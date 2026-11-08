// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Assign } from "./Assign";
import type { CaptionAnimation } from "./CaptionAnimation";
import type { CaptionBox } from "./CaptionBox";
import type { CaptionOutline } from "./CaptionOutline";
import type { CaptionShadow } from "./CaptionShadow";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { TextCase } from "./TextCase";
import type { Vec2 } from "./Vec2";

/**
 * Changes of a caption style; `null` (or absent) fields are unchanged.
 */
export type CaptionStylePatch = { 
/**
 * Another origin preset (`{"value": null}` clears it).
 */
preset?: Assign<string | null>, 
/**
 * Another font family.
 */
fontFamily?: string, 
/**
 * Another font weight (1–1000).
 */
fontWeight?: number, 
/**
 * Italic or not.
 */
italic?: boolean, 
/**
 * Another font size (> 0).
 */
fontSize?: DesignPixels, 
/**
 * Another letter case.
 */
textCase?: TextCase, 
/**
 * Another line height, in `(0, 10]`.
 */
lineHeight?: number, 
/**
 * Another colour of words not spoken yet.
 */
textColor?: Color, 
/**
 * Another colour of the active word.
 */
activeColor?: Color, 
/**
 * Another colour of emphasised words.
 */
emphasisColor?: Color, 
/**
 * Another outline.
 */
outline?: CaptionOutline, 
/**
 * Another shadow.
 */
shadow?: CaptionShadow, 
/**
 * Another plate.
 */
background?: CaptionBox, 
/**
 * Another position of the block's bottom centre.
 */
position?: Vec2, 
/**
 * Another animation.
 */
animation?: CaptionAnimation, 
/**
 * Whether spoken words keep the active colour.
 */
highlightSpoken?: boolean, 
/**
 * Another longest line, in characters.
 */
maxCharsPerLine?: number, 
/**
 * Another number of lines shown at once.
 */
maxLines?: number, };
