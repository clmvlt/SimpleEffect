import type { FrameRate } from '@se/protocol';
import { Field, Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { frameRateFromValue, frameRateValue } from '../format-choices';
import { framesPerSecond } from '../format-label';

/** Props of {@link FrameRateSelect}. */
export interface FrameRateSelectProps {
  /** The rates the core supports (`NewProjectDefaults.frameRates`). */
  readonly rates: readonly FrameRate[];
  readonly value: FrameRate;
  readonly onChange: (rate: FrameRate) => void;
  /** Shown under the field, e.g. why it is disabled. */
  readonly description?: string | undefined;
  readonly disabled?: boolean | undefined;
}

/** Picks a frame rate, shown as « 29,97 i/s » in the interface language. */
export function FrameRateSelect({
  rates,
  value,
  onChange,
  description,
  disabled = false,
}: FrameRateSelectProps) {
  const { t, i18n } = useTranslation();
  const numbers = new Intl.NumberFormat(i18n.language, { maximumFractionDigits: 3 });
  const listed = rates.some((rate) => frameRateValue(rate) === frameRateValue(value));
  const choices = listed ? rates : [value, ...rates];
  return (
    <Select
      value={frameRateValue(value)}
      disabled={disabled}
      onValueChange={(chosen) => {
        const rate = frameRateFromValue(choices, chosen);
        if (rate !== undefined) onChange(rate);
      }}
    >
      <Field label={t('format.frameRate')} description={description}>
        <SelectTrigger className="w-full">
          <SelectValue />
        </SelectTrigger>
      </Field>
      <SelectContent>
        {choices.map((rate) => (
          <SelectItem key={frameRateValue(rate)} value={frameRateValue(rate)}>
            {t('format.frameRateValue', { value: numbers.format(framesPerSecond(rate)) })}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
