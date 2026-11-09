import type { LibraryBrandKit } from '@se/protocol';
import { PropertyRow } from '@se/ui';
import { useTranslation } from 'react-i18next';

import {
  BrandColorField,
  PALETTE_ROLES,
  brandFonts,
  brandSwatches,
} from '../../../shared/brand-kit';
import { FontFamilySelect } from '../../../shared/fonts';
import { withColor, withFont } from '../services/kit-draft';

/** Props of the kit editor's field groups. */
export interface KitFieldsProps {
  readonly kit: LibraryBrandKit;
  readonly onChange: (kit: LibraryBrandKit) => void;
}

/** The kit's two fonts and five colours. */
export function KitIdentityFields({ kit, onChange }: KitFieldsProps) {
  const { t } = useTranslation();
  const palette = brandSwatches(kit).map(({ role, color }) => ({
    color,
    label: t(`brandKit.roles.${role}`),
  }));
  return (
    <>
      <section aria-labelledby="kit-fonts" className="flex flex-col gap-1">
        <h4 id="kit-fonts" className="text-xs font-medium text-text-muted">
          {t('brandKit.settings.fonts')}
        </h4>
        <PropertyRow label={t('brandKit.roles.heading')} htmlFor="kit-heading">
          <FontFamilySelect
            id="kit-heading"
            value={kit.fonts.heading}
            brandFonts={[]}
            onChange={(heading) => {
              onChange(withFont(kit, 'heading', heading));
            }}
          />
        </PropertyRow>
        <PropertyRow label={t('brandKit.roles.body')} htmlFor="kit-body">
          <FontFamilySelect
            id="kit-body"
            value={kit.fonts.body}
            brandFonts={brandFonts({ ...kit, fonts: { ...kit.fonts, body: kit.fonts.heading } })}
            onChange={(body) => {
              onChange(withFont(kit, 'body', body));
            }}
          />
        </PropertyRow>
      </section>
      <section aria-labelledby="kit-palette" className="flex flex-col gap-1">
        <h4 id="kit-palette" className="text-xs font-medium text-text-muted">
          {t('brandKit.settings.palette')}
        </h4>
        {PALETTE_ROLES.map((role) => (
          <PropertyRow key={role} label={t(`brandKit.roles.${role}`)} htmlFor={`kit-${role}`}>
            <BrandColorField
              id={`kit-${role}`}
              label={t(`brandKit.roles.${role}`)}
              value={kit.colors[role].slice(0, 7)}
              palette={palette}
              onChange={(color) => {
                onChange(withColor(kit, role, color));
              }}
            />
          </PropertyRow>
        ))}
      </section>
    </>
  );
}
