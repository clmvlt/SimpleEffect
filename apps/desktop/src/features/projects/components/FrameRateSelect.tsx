import type { FrameRate } from '@se/protocol';
import { Field, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { frameRateFromValue, frameRateValue } from '../services/form-choices';
import { framesPerSecond } from '../services/format-label';

/** Props of {@link FrameRateSelect}. */
export interface FrameRateSelectProps {
  /** The rates the core supports (`NewProjectDefaults.frameRates`). */
  readonly rates: readonly FrameRate[];
  readonly value: FrameRate;
  readonly onChange: (rate: FrameRate) => void;
}

/** Picks a frame rate, shown as « 29,97 i/s » in the interface language. */
export function FrameRateSelect({ rates, value, onChange }: FrameRateSelectProps) {
  const { t, i18n } = useTranslation();
  const numbers = new Intl.NumberFormat(i18n.language, { maximumFractionDigits: 3 });
  return (
    <Select
      value={frameRateValue(value)}
      onValueChange={(chosen) => {
        const rate = frameRateFromValue(rates, chosen);
        if (rate !== undefined) onChange(rate);
      }}
    >
      <Field label={t('projects.newDialog.frameRate')}>
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
      </Field>
      <SelectContent>
        {rates.map((rate) => (
          <SelectItem key={frameRateValue(rate)} value={frameRateValue(rate)}>
            {t('projects.newDialog.frameRateValue', {
              value: numbers.format(framesPerSecond(rate)),
            })}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
