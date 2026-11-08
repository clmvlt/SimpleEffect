import type { EffortLevel, ProviderCapabilities } from '@se/protocol';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { updateComposer, useComposer } from '../store/composer-store';

/** Value of the « default of the settings » choice. */
const DEFAULT = '__default';

/** Props of {@link ModelSelect} and {@link EffortSelect}. */
export interface ChoiceSelectProps {
  readonly capabilities: ProviderCapabilities | null;
  readonly disabled: boolean;
}

/** The model of the next task (the provider's models; « Par défaut » follows the settings). */
export function ModelSelect({ capabilities, disabled }: ChoiceSelectProps) {
  const { t } = useTranslation();
  const model = useComposer((state) => state.model);
  const models = capabilities?.models ?? [];
  if (models.length === 0) return null;
  return (
    <Select
      value={model ?? DEFAULT}
      disabled={disabled}
      onValueChange={(value) => {
        updateComposer({ model: value === DEFAULT ? null : value });
      }}
    >
      <SelectTrigger size="xs" aria-label={t('ai.composer.model')} className="w-28">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={DEFAULT}>{t('ai.composer.modelDefault')}</SelectItem>
        {model !== null && !models.some((option) => option.id === model) && (
          <SelectItem value={model}>{model}</SelectItem>
        )}
        {models.map((option) => (
          <SelectItem key={option.id} value={option.id}>
            {option.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}

/** The effort of the next task (the provider's levels; « Par défaut » follows the settings). */
export function EffortSelect({ capabilities, disabled }: ChoiceSelectProps) {
  const { t } = useTranslation();
  const effort = useComposer((state) => state.effort);
  const levels = capabilities?.effortLevels ?? [];
  if (levels.length === 0) return null;
  return (
    <Select
      value={effort ?? DEFAULT}
      disabled={disabled}
      onValueChange={(value) => {
        updateComposer({ effort: levels.find((level) => level === value) ?? null });
      }}
    >
      <SelectTrigger size="xs" aria-label={t('ai.composer.effort')} className="w-28">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        <SelectItem value={DEFAULT}>{t('ai.composer.effortDefault')}</SelectItem>
        {levels.map((level: EffortLevel) => (
          <SelectItem key={level} value={level}>
            {t(`settings.ai.effort.${level}`)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
}
