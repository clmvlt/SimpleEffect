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

import { useFontFamilies } from '../../../shared/fonts';

/** Whether `family` is in the font catalogue (always true until the catalogue is listed). */
export function useKnownFamily(family: string): boolean {
  const families = useFontFamilies();
  return families.length === 0 || families.some((entry) => entry.name === family);
}

/**
 * A font family menu over the core's catalogue (bundled fonts, then installed ones); a family the
 * catalogue lacks stays shown as the current value.
 */
export function FontFamilySelect({
  id,
  value,
  onChange,
}: {
  readonly id: string;
  readonly value: string;
  readonly onChange: (family: string) => void;
}) {
  const { t } = useTranslation();
  const families = useFontFamilies();
  const known = useKnownFamily(value);
  const bundled = families.filter((family) => family.source === 'bundled');
  const installed = families.filter((family) => family.source === 'system');
  return (
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger id={id} size="xs" className="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {!known && <SelectItem value={value}>{value}</SelectItem>}
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
