// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { BrandFonts } from "./BrandFonts";
import type { BrandKitSource } from "./BrandKitSource";
import type { BrandLogo } from "./BrandLogo";
import type { BrandPalette } from "./BrandPalette";
import type { CaptionStyle } from "./CaptionStyle";
import type { LibraryTemplateRef } from "./LibraryTemplateRef";
import type { TextStyle } from "./TextStyle";

/**
 * A brand identity: fonts, a named palette, logos and the default looks of new texts and
 * captions. Every value is static (no keyframes): it is a starting point, copied into what it
 * styles.
 */
export type BrandKit = { 
/**
 * The library kit this copy comes from, and its version then; `null` for a kit that exists
 * only in this project (made before prompt 25, or whose library entry is gone).
 */
source: BrandKitSource | null, 
/**
 * Name shown in the interface (« Acme », « Chaîne perso »).
 */
name: string, 
/**
 * The two font roles.
 */
fonts: BrandFonts, 
/**
 * The five colour roles.
 */
colors: BrandPalette, 
/**
 * Logos, as image assets of the project (the first one is the main logo).
 */
logos: Array<BrandLogo>, 
/**
 * Style of new texts (static values).
 */
textStyle: TextStyle, 
/**
 * Look of new caption styles; its `id` is not used (new styles get their own).
 */
captionStyle: CaptionStyle, 
/**
 * Favourite intro templates of the library, most preferred first.
 */
introTemplates: Array<LibraryTemplateRef>, 
/**
 * Favourite outro templates of the library, most preferred first.
 */
outroTemplates: Array<LibraryTemplateRef>, };
