// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Assign } from "./Assign";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { TextAlign } from "./TextAlign";
import type { TextCase } from "./TextCase";
import type { Vec2 } from "./Vec2";

/**
 * Changes of a [`crate::TextItem`]: its content and the static parts of its style (animatable
 * ones go through [`super::SetKeyframes`]; `fontSize` and `color` here set a static value).
 */
export type TextPatch = { 
/**
 * Another text.
 */
content?: string, 
/**
 * Another font family.
 */
fontFamily?: string, 
/**
 * Another weight (1 to 1000).
 */
fontWeight?: number, 
/**
 * Italic or upright.
 */
italic?: boolean, 
/**
 * A static font size (replaces keyframes).
 */
fontSize?: DesignPixels, 
/**
 * A static glyph colour (replaces keyframes).
 */
color?: Color, 
/**
 * Another line height (multiple of the font size).
 */
lineHeight?: number, 
/**
 * Another alignment.
 */
align?: TextAlign, 
/**
 * Another wrapping width, or `{"value": null}` for no wrapping.
 */
maxWidth?: Assign<DesignPixels | null>, 
/**
 * Another letter case.
 */
textCase?: TextCase, 
/**
 * Outline on or off.
 */
strokeEnabled?: boolean, 
/**
 * Shadow on or off.
 */
shadowEnabled?: boolean, 
/**
 * Background plate on or off.
 */
backgroundEnabled?: boolean, 
/**
 * Another plate padding (design pixels, x and y).
 */
backgroundPadding?: Vec2, 
/**
 * Another plate corner radius.
 */
backgroundCornerRadius?: DesignPixels, };
