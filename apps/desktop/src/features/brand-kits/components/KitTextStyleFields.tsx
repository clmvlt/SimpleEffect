import {
  PropertyRow,
  ScrubInput,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Switch,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { BrandColorField, brandFonts, brandSwatches } from '../../../shared/brand-kit';
import { FontFamilySelect } from '../../../shared/fonts';
import { constant, staticValue, withTextSize, withTextStyle } from '../services/kit-draft';
import type { KitFieldsProps } from './KitIdentityFields';

/** Weights offered for new texts. */
const WEIGHTS = [300, 400, 500, 600, 700, 800, 900] as const;

/** A colour without its alpha (`#rrggbb`), `fallback` when the field holds none. */
const opaque = (color: string | null, fallback: string) => (color ?? fallback).slice(0, 7);

/** The look of new texts: font, weight, size, colour, capitals, outline, shadow, plate. */
export function KitTextStyleFields({ kit, onChange }: KitFieldsProps) {
  const { t } = useTranslation();
  const style = kit.textStyle;
  const palette = brandSwatches(kit).map(({ role, color }) => ({
    color,
    label: t(`brandKit.roles.${role}`),
  }));
  const strokeColor = opaque(staticValue(style.stroke.color), kit.colors.background);
  const plateColor = opaque(staticValue(style.background.color), kit.colors.background);
  return (
    <section aria-labelledby="kit-text" className="flex flex-col gap-1">
      <h4 id="kit-text" className="text-xs font-medium text-text-muted">
        {t('brandKit.settings.textStyle')}
      </h4>
      <PropertyRow label={t('brandKit.settings.font')} htmlFor="kit-text-font">
        <FontFamilySelect
          id="kit-text-font"
          value={style.fontFamily}
          brandFonts={brandFonts(kit)}
          onChange={(fontFamily) => {
            onChange(withTextStyle(kit, { fontFamily }));
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.weight')} htmlFor="kit-text-weight">
        <Select
          value={String(style.fontWeight)}
          onValueChange={(value) => {
            onChange(withTextStyle(kit, { fontWeight: Number(value) }));
          }}
        >
          <SelectTrigger id="kit-text-weight" size="xs" className="w-full">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {WEIGHTS.map((weight) => (
              <SelectItem key={weight} value={String(weight)}>
                {weight}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.size')} htmlFor="kit-text-size">
        <ScrubInput
          id="kit-text-size"
          value={staticValue(style.fontSize) ?? 96}
          min={8}
          max={600}
          step={1}
          unit="px"
          onChange={(size) => {
            onChange(withTextSize(kit, size));
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.color')} htmlFor="kit-text-color">
        <BrandColorField
          id="kit-text-color"
          label={t('brandKit.settings.color')}
          value={opaque(staticValue(style.color), kit.colors.text)}
          palette={palette}
          onChange={(color) => {
            onChange(withTextStyle(kit, { color: constant(color) }));
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.uppercase')} htmlFor="kit-text-case">
        <Switch
          id="kit-text-case"
          checked={style.textCase === 'uppercase'}
          onCheckedChange={(checked) => {
            onChange(withTextStyle(kit, { textCase: checked ? 'uppercase' : 'normal' }));
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.outline')}>
        <div className="flex items-center gap-2">
          <Switch
            checked={style.stroke.enabled}
            aria-label={t('brandKit.settings.outline')}
            onCheckedChange={(enabled) => {
              onChange(withTextStyle(kit, { stroke: { ...style.stroke, enabled } }));
            }}
          />
          <BrandColorField
            label={t('brandKit.settings.outlineColor')}
            value={strokeColor}
            palette={palette}
            disabled={!style.stroke.enabled}
            onChange={(color) => {
              onChange(withTextStyle(kit, { stroke: { ...style.stroke, color: constant(color) } }));
            }}
          />
        </div>
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.shadow')} htmlFor="kit-text-shadow">
        <Switch
          id="kit-text-shadow"
          checked={style.shadow.enabled}
          onCheckedChange={(enabled) => {
            onChange(withTextStyle(kit, { shadow: { ...style.shadow, enabled } }));
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('brandKit.settings.plate')}>
        <div className="flex items-center gap-2">
          <Switch
            checked={style.background.enabled}
            aria-label={t('brandKit.settings.plate')}
            onCheckedChange={(enabled) => {
              onChange(withTextStyle(kit, { background: { ...style.background, enabled } }));
            }}
          />
          <BrandColorField
            label={t('brandKit.settings.plateColor')}
            value={plateColor}
            palette={palette}
            disabled={!style.background.enabled}
            onChange={(color) => {
              onChange(
                withTextStyle(kit, {
                  background: { ...style.background, color: constant(color) },
                }),
              );
            }}
          />
        </div>
      </PropertyRow>
    </section>
  );
}
