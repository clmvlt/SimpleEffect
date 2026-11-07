// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { Stroke } from "./Stroke";
import type { TextAlign } from "./TextAlign";
import type { TextBackground } from "./TextBackground";
import type { TextCase } from "./TextCase";
import type { TextShadow } from "./TextShadow";

/**
 * How a text looks. Sizes are design pixels (ADR-0015); the properties that change over time
 * (size, colours, spacing, stroke, shadow, background colour) are [`Animatable`].
 */
export type TextStyle = { 
/**
 * Font family name, as listed by the font catalogue (bundled or system fonts). A missing
 * family is drawn with the default font and reported.
 */
fontFamily: string, 
/**
 * Font weight, 1 to 1000 (400 regular, 700 bold).
 */
fontWeight: number, 
/**
 * Italic (or oblique) face.
 */
italic: boolean, 
/**
 * Font size (the em size) in design pixels; > 0.
 */
fontSize: Animatable<DesignPixels>, 
/**
 * Colour of the glyphs.
 */
color: Animatable<Color>, 
/**
 * Outline around the glyphs.
 */
stroke: Stroke, 
/**
 * Drop shadow under the glyphs (and their outline).
 */
shadow: TextShadow, 
/**
 * Plate drawn behind the text block.
 */
background: TextBackground, 
/**
 * Extra space between characters, in design pixels (negative tightens).
 */
letterSpacing: Animatable<DesignPixels>, 
/**
 * Distance between baselines as a multiple of the font size (1.2 by default); > 0.
 */
lineHeight: number, 
/**
 * Alignment of the lines in the text block.
 */
align: TextAlign, 
/**
 * Width at which lines wrap, in design pixels; `None` keeps each line on one line.
 */
maxWidth: DesignPixels | null, 
/**
 * Letter case shown (the content keeps its own case).
 */
textCase: TextCase, };
