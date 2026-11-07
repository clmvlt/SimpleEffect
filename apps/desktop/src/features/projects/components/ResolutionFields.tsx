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

import type { useNewProjectForm } from '../hooks/use-new-project-form';
import { CUSTOM_RESOLUTION, resolutionValue } from '../services/form-choices';

/** Props of {@link ResolutionFields}. */
export interface ResolutionFieldsProps {
  readonly form: ReturnType<typeof useNewProjectForm>;
}

/** The resolution choice of the new project form, with width and height when custom. */
export function ResolutionFields({ form }: ResolutionFieldsProps) {
  const { t } = useTranslation();
  const { values, resolutions, customSize, errors } = form;
  const sizeError = errors.size && t(`projects.newDialog.errors.${errors.size}`);
  return (
    <div className="flex flex-col gap-2">
      <Select
        value={customSize ? CUSTOM_RESOLUTION : resolutionValue(values)}
        onValueChange={(value) => {
          const chosen = resolutions.find((resolution) => resolutionValue(resolution) === value);
          form.chooseResolution(chosen ?? 'custom');
        }}
      >
        <Field label={t('projects.newDialog.resolution')}>
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
          <SelectItem value={CUSTOM_RESOLUTION}>
            {t('projects.newDialog.customResolution')}
          </SelectItem>
        </SelectContent>
      </Select>
      {customSize && (
        <div className="flex gap-2">
          <Field className="flex-1" label={t('projects.newDialog.width')} error={sizeError}>
            <Input
              type="number"
              min={2}
              step={2}
              value={values.width}
              onChange={(event) => {
                form.setSize({ width: event.target.valueAsNumber });
              }}
            />
          </Field>
          <Field className="flex-1" label={t('projects.newDialog.height')}>
            <Input
              type="number"
              min={2}
              step={2}
              value={values.height}
              onChange={(event) => {
                form.setSize({ height: event.target.valueAsNumber });
              }}
            />
          </Field>
        </div>
      )}
    </div>
  );
}
