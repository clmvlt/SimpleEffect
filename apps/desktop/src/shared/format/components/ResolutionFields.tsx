import {
  Field,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { CUSTOM_RESOLUTION, resolutionValue } from '../format-choices';
import type { FormatDraft } from '../format-draft';
import type { Resolution } from '../resolutions';

/** Props of {@link ResolutionFields}. */
export interface ResolutionFieldsProps {
  readonly value: FormatDraft;
  /** The resolutions offered for the draft's ratio. */
  readonly resolutions: readonly Resolution[];
  /** Whether the width and height fields show. */
  readonly customSize: boolean;
  readonly onChoose: (resolution: Resolution | 'custom') => void;
  readonly onSize: (size: Partial<Resolution>) => void;
  /** Shown under the width when the size is refused. */
  readonly error?: string | undefined;
}

/** The resolution choice of a format, with width and height when custom. */
export function ResolutionFields({
  value,
  resolutions,
  customSize,
  onChoose,
  onSize,
  error,
}: ResolutionFieldsProps) {
  const { t } = useTranslation();
  return (
    <div className="flex flex-col gap-2">
      <Select
        value={customSize ? CUSTOM_RESOLUTION : resolutionValue(value)}
        onValueChange={(chosen) => {
          const resolution = resolutions.find((entry) => resolutionValue(entry) === chosen);
          onChoose(resolution ?? 'custom');
        }}
      >
        <Field label={t('format.resolution')}>
          <SelectTrigger className="w-full">
            <SelectValue />
          </SelectTrigger>
        </Field>
        <SelectContent>
          {resolutions.map((resolution) => (
            <SelectItem key={resolutionValue(resolution)} value={resolutionValue(resolution)}>
              {`${String(resolution.width)} × ${String(resolution.height)}`}
            </SelectItem>
          ))}
          <SelectItem value={CUSTOM_RESOLUTION}>{t('format.customResolution')}</SelectItem>
        </SelectContent>
      </Select>
      {customSize && (
        <div className="flex gap-2">
          <Field className="flex-1" label={t('format.width')} error={error}>
            <Input
              type="number"
              min={2}
              step={2}
              value={value.width}
              onChange={(event) => {
                onSize({ width: event.target.valueAsNumber });
              }}
            />
          </Field>
          <Field className="flex-1" label={t('format.height')}>
            <Input
              type="number"
              min={2}
              step={2}
              value={value.height}
              onChange={(event) => {
                onSize({ height: event.target.valueAsNumber });
              }}
            />
          </Field>
        </div>
      )}
    </div>
  );
}
