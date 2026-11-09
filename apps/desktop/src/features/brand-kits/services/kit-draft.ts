import { captionPreset, type CaptionPresetId } from '@se/engine/caption-styles';
import type {
  Animatable,
  CaptionStyle,
  LibraryBrandKit,
  LibraryTemplate,
  LibraryTemplateRef,
  TextStyle,
} from '@se/protocol';

import type { PaletteRole } from '../../../shared/brand-kit';

/**
 * Pure edits of a brand kit being edited in the settings (a draft saved by « Enregistrer le
 * kit »). Kits hold static values only: animatable fields are read and written as constants.
 */

/** The value of a kit's static field (the first keyframe if a kit ever held some). */
export function staticValue<T>(value: Animatable<T>): T | null {
  if (value.kind === 'static') return value.value;
  return value.keyframes[0]?.value ?? null;
}

/** A static value. */
export function constant<T>(value: T): Animatable<T> {
  return { kind: 'static', value };
}

/** The kit's text style with `patch` applied. */
export function withTextStyle(kit: LibraryBrandKit, patch: Partial<TextStyle>): LibraryBrandKit {
  return { ...kit, textStyle: { ...kit.textStyle, ...patch } };
}

/** The kit with a new font size of new texts (design pixels). */
export function withTextSize(kit: LibraryBrandKit, size: number): LibraryBrandKit {
  return withTextStyle(kit, { fontSize: constant(size) });
}

/** Which kit font. */
export type KitFontRole = 'heading' | 'body';

/**
 * The kit with its `role` font changed. The looks of new texts and captions follow when they
 * used the old font (they start from the kit's fonts), so the font chosen shows at once; a look
 * given another font keeps it.
 */
export function withFont(kit: LibraryBrandKit, role: KitFontRole, family: string): LibraryBrandKit {
  const previous = kit.fonts[role];
  const follow = (current: string) => (current === previous ? family : current);
  return {
    ...kit,
    fonts: { ...kit.fonts, [role]: family },
    textStyle: { ...kit.textStyle, fontFamily: follow(kit.textStyle.fontFamily) },
    captionStyle: { ...kit.captionStyle, fontFamily: follow(kit.captionStyle.fontFamily) },
  };
}

/** The kit with one palette colour changed. */
export function withColor(kit: LibraryBrandKit, role: PaletteRole, color: string): LibraryBrandKit {
  return { ...kit, colors: { ...kit.colors, [role]: color } };
}

/** The kit's caption look with `patch` applied. */
export function withCaptionStyle(
  kit: LibraryBrandKit,
  patch: Partial<CaptionStyle>,
): LibraryBrandKit {
  return { ...kit, captionStyle: { ...kit.captionStyle, ...patch } };
}

/** The kit's caption look reset to preset `preset`, keeping its id and the kit's body font. */
export function withCaptionPreset(kit: LibraryBrandKit, preset: CaptionPresetId): LibraryBrandKit {
  const { style } = captionPreset(preset);
  return {
    ...kit,
    captionStyle: { ...style, id: kit.captionStyle.id, preset, fontFamily: kit.fonts.body },
  };
}

/** The kit with logo `file` renamed. */
export function withLogoName(kit: LibraryBrandKit, file: string, name: string): LibraryBrandKit {
  return {
    ...kit,
    logos: kit.logos.map((logo) => (logo.file === file ? { ...logo, name } : logo)),
  };
}

/** The kit without logo `file`. */
export function withoutLogo(kit: LibraryBrandKit, file: string): LibraryBrandKit {
  return { ...kit, logos: kit.logos.filter((logo) => logo.file !== file) };
}

/**
 * The draft once logos were added on the server (`saved` holds them, with the stored values of
 * everything else): the draft's own edits stay, the new logos are appended, and the version
 * follows the saved one.
 */
export function withAddedLogos(draft: LibraryBrandKit, saved: LibraryBrandKit): LibraryBrandKit {
  const known = new Set(draft.logos.map((logo) => logo.file));
  return {
    ...draft,
    version: saved.version,
    updatedAt: saved.updatedAt,
    logos: [...draft.logos, ...saved.logos.filter((logo) => !known.has(logo.file))],
  };
}

/** Which favourite list. */
export type FavoriteList = 'introTemplates' | 'outroTemplates';

/** The kit with library template `entry` added to a favourite list (once). */
export function withFavorite(
  kit: LibraryBrandKit,
  list: FavoriteList,
  entry: LibraryTemplate,
): LibraryBrandKit {
  if (kit[list].some((favorite) => favorite.id === entry.id)) return kit;
  const favorite: LibraryTemplateRef = { id: entry.id, name: entry.name };
  return { ...kit, [list]: [...kit[list], favorite] };
}

/** The kit without favourite `id` in `list`. */
export function withoutFavorite(
  kit: LibraryBrandKit,
  list: FavoriteList,
  id: string,
): LibraryBrandKit {
  return { ...kit, [list]: kit[list].filter((favorite) => favorite.id !== id) };
}

/** Whether `draft` differs from `saved` in anything the user edits. */
export function isDirty(draft: LibraryBrandKit, saved: LibraryBrandKit): boolean {
  const editable = (kit: LibraryBrandKit) => JSON.stringify({ ...kit, version: 0, updatedAt: 0 });
  return editable(draft) !== editable(saved);
}
