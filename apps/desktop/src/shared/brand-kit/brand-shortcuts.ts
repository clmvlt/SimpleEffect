import type { BrandKit, BrandRole } from '@se/protocol';

/** The palette roles in the order fields show them. */
export const PALETTE_ROLES = ['primary', 'secondary', 'accent', 'text', 'background'] as const;

/** A palette role. */
export type PaletteRole = (typeof PALETTE_ROLES)[number];

/** A named colour of the kit, for colour fields (`#rrggbb`, the alpha dropped). */
export interface BrandSwatch {
  readonly role: PaletteRole;
  readonly color: string;
}

/** The kit's five colours as swatches (none without a kit; library kits fit too). */
export function brandSwatches(
  kit: Pick<BrandKit, 'colors'> | null | undefined,
): readonly BrandSwatch[] {
  if (kit === null || kit === undefined) return [];
  return PALETTE_ROLES.map((role) => ({ role, color: kit.colors[role].slice(0, 7) }));
}

/** The kit's font families, titles first, without duplicates (none without a kit). */
export function brandFonts(kit: Pick<BrandKit, 'fonts'> | null | undefined): readonly string[] {
  if (kit === null || kit === undefined) return [];
  const { heading, body } = kit.fonts;
  return heading === body ? [heading] : [heading, body];
}

/** Whether `role` is a palette role (template parameters may also take font and logo roles). */
export function isPaletteRole(role: BrandRole): role is PaletteRole {
  return (PALETTE_ROLES as readonly string[]).includes(role);
}
