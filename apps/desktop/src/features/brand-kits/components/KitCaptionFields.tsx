import { CAPTION_STYLE_PRESETS } from '@se/engine/caption-styles';
import { PropertyRow, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { BrandColorField, brandFonts, brandSwatches } from '../../../shared/brand-kit';
import { FontFamilySelect } from '../../../shared/fonts';
import { withCaptionPreset, withCaptionStyle } from '../services/kit-draft';
import type { KitFieldsProps } from './KitIdentityFields';

/**
 * The look of new captions: a predefined style to start from (its font replaced by the kit's
 * body font), then the font and the two main colours. Finer settings are made on the captions
 * themselves (« Sous-titres → Style ») and saved to the library.
 */
export function KitCaptionFields({ kit, onChange }: KitFieldsProps) {
  const { t } = useTranslation();
  const style = kit.captionStyle;
  const palette = brandSwatches(kit).map(({ role, color }) => ({
    color,
    label: t(`brandKit.roles.${role}`),
  }));
  const colour = (field: 'textColor' | 'activeColor', label: string) => (
    <PropertyRow label={label} htmlFor={`kit-caption-${field}`}>
      <BrandColorField
        id={`kit-caption-${field}`}
        label={label}
        value={style[field].slice(0, 7)}
        palette={palette}
        onChange={(color) => {
          onChange(withCaptionStyle(kit, { [field]: color }));
        }}
      />
    </PropertyRow>
  );
  return (
    <section aria-labelledby="kit-caption" className="flex flex-col gap-1">
      <h4 id="kit-caption" className="text-xs font-medium text-text-muted">
        {t('brandKit.settings.captionStyle')}
      </h4>
      <PropertyRow label={t('brandKit.settings.captionPreset')} htmlFor="kit-caption-preset">
        <Select
          value={style.preset ?? ''}
          onValueChange={(value) => {
            const preset = CAPTION_STYLE_PRESETS.find((candidate) => candidate.id === value);
            if (preset !== undefined) onChange(withCaptionPreset(kit, preset.id));
          }}
        >
          <SelectTrigger id="kit-caption-preset" size="xs" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {CAPTION_STYLE_PRESETS.map((preset) => (
              <SelectItem key={preset.id} value={preset.id}>
                {t(`editor.captions.presets.${preset.id}`)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.font')} htmlFor="kit-caption-font">
        <FontFamilySelect
          id="kit-caption-font"
          value={style.fontFamily}
          brandFonts={brandFonts(kit)}
          onChange={(fontFamily) => {
            onChange(withCaptionStyle(kit, { fontFamily }));
          }}
        />
      </PropertyRow>
      {colour('textColor', t('brandKit.settings.textColor'))}
      {colour('activeColor', t('brandKit.settings.activeColor'))}
    </section>
  );
}
