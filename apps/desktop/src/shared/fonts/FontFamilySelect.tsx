import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectTrigger,
  SelectValue,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useBrandShortcuts } from '../brand-kit';
import { useFontFamilies } from './use-font-families';

/** Whether `family` is in the font catalogue (always true until the catalogue is listed). */
export function useKnownFamily(family: string): boolean {
  const families = useFontFamilies();
  return families.length === 0 || families.some((entry) => entry.name === family);
}

/**
 * A font family menu over the core's catalogue: the brand kit's fonts first (prompt 25), then
 * bundled fonts, then installed ones; a family the catalogue lacks stays shown as the current
 * value. `brandFonts` overrides the open project's kit (the kit editor shows its own draft).
 */
export function FontFamilySelect({
  id,
  value,
  onChange,
  brandFonts,
}: {
  readonly id: string;
  readonly value: string;
  readonly onChange: (family: string) => void;
  readonly brandFonts?: readonly string[] | undefined;
}) {
  const { t } = useTranslation();
  const families = useFontFamilies();
  const shortcuts = useBrandShortcuts();
  const known = useKnownFamily(value);
  const brand = brandFonts ?? shortcuts.fonts;
  const listed = (family: { readonly name: string }) => !brand.includes(family.name);
  const bundled = families.filter((family) => family.source === 'bundled' && listed(family));
  const installed = families.filter((family) => family.source === 'system' && listed(family));
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id} size="xs" className="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {!known && !brand.includes(value) && <SelectItem value={value}>{value}</SelectItem>}
        {brand.length > 0 && (
          <SelectGroup>
            <SelectLabel>{t('brandKit.fontsGroup')}</SelectLabel>
            {brand.map((family) => (
              <SelectItem key={family} value={family}>
                {family}
              </SelectItem>
            ))}
          </SelectGroup>
        )}
        <SelectGroup>
          <SelectLabel>{t('editor.inspector.text.bundledFonts')}</SelectLabel>
          {bundled.map((family) => (
            <SelectItem key={family.name} value={family.name}>
              {family.name}
            </SelectItem>
          ))}
        </SelectGroup>
        {installed.length > 0 && (
          <SelectGroup>
            <SelectLabel>{t('editor.inspector.text.systemFonts')}</SelectLabel>
            {installed.map((family) => (
              <SelectItem key={family.name} value={family.name}>
                {family.name}
              </SelectItem>
            ))}
          </SelectGroup>
        )}
      </SelectContent>
    </Select>
  );
}
