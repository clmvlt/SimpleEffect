// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { BrandFonts } from "./BrandFonts";
import type { BrandKitId } from "./BrandKitId";
import type { BrandPalette } from "./BrandPalette";
import type { CaptionStyle } from "./CaptionStyle";
import type { LibraryLogo } from "./LibraryLogo";
import type { LibraryTemplateRef } from "./LibraryTemplateRef";
import type { TextStyle } from "./TextStyle";

/**
 * A brand kit of the app's library.
 */
export type LibraryBrandKit = { 
/**
 * Stable id (project copies name it as their source).
 */
id: BrandKitId, 
/**
 * Bumped by each save; project copies holding an older version are offered the update.
 */
version: number, 
/**
 * When it last changed (Unix milliseconds).
 */
updatedAt: number, 
/**
 * Name shown in the interface.
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
 * Logos, the first being the main one; their files live in the kit's folder (served at
 * `/media/brand-kits/{kitId}/{file}`).
 */
logos: Array<LibraryLogo>, 
/**
 * Style of new texts (static values).
 */
textStyle: TextStyle, 
/**
 * Look of new caption styles.
 */
captionStyle: CaptionStyle, 
/**
 * Favourite intro templates of the library.
 */
introTemplates: Array<LibraryTemplateRef>, 
/**
 * Favourite outro templates of the library.
 */
outroTemplates: Array<LibraryTemplateRef>, };
