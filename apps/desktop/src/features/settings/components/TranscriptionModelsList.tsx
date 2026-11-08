import type { TranscriptionStatus } from '@se/protocol';
import { Button, Label, RadioGroup, RadioGroupItem } from '@se/ui';
import { useId } from 'react';
import { useTranslation } from 'react-i18next';

import type { TranscriptionActions } from '../hooks/use-transcription-settings';
import {
  AUTOMATIC_MODEL,
  downloadsVoiceModelToo,
  hasInstalledModel,
  modelChoiceValue,
  modelChoices,
  modelFromChoice,
} from '../services/transcription-models';
import { TranscriptionModelRow } from './TranscriptionModelRow';

/** Props of {@link TranscriptionModelsList}. */
export interface TranscriptionModelsListProps {
  readonly status: TranscriptionStatus;
  readonly actions: TranscriptionActions;
}

/**
 * « Modèles »: the catalogue and imported models (download, progress, removal), the model used
 * (« Automatique (recommandé) » or an installed one) and « Importer un modèle… ».
 */
export function TranscriptionModelsList({ status, actions }: TranscriptionModelsListProps) {
  const { t } = useTranslation();
  const voiceModelSize = downloadsVoiceModelToo(status) ? status.voiceModel.size : null;
  return (
    <section aria-labelledby="settings-transcription-models" className="flex flex-col gap-3">
      <h3 id="settings-transcription-models" className="text-sm font-semibold text-text">
        {t('settings.transcription.models.title')}
      </h3>
      <ul
        aria-labelledby="settings-transcription-models"
        className="flex flex-col divide-y divide-border"
      >
        {status.models.map((model) => (
          <TranscriptionModelRow
            key={model.id}
            model={model}
            voiceModelSize={voiceModelSize}
            actions={actions}
          />
        ))}
      </ul>
      <div>
        <Button
          size="xs"
          variant="secondary"
          loading={actions.importing}
          onClick={actions.importModel}
        >
          {t('settings.transcription.models.import')}
        </Button>
      </div>
      <ModelChoice status={status} onSave={actions.save} />
    </section>
  );
}

function ModelChoice({
  status,
  onSave,
}: {
  readonly status: TranscriptionStatus;
  readonly onSave: TranscriptionActions['save'];
}) {
  const { t } = useTranslation();
  const id = useId();
  const choices = modelChoices(status);
  return (
    <div className="flex flex-col gap-1.5">
      <span id={`${id}-label`} className="text-xs font-medium text-text">
        {t('settings.transcription.models.choice')}
      </span>
      <RadioGroup
        aria-labelledby={`${id}-label`}
        value={modelChoiceValue(status)}
        onValueChange={(value) => {
          onSave({ model: modelFromChoice(value) });
        }}
      >
        {[
          { value: AUTOMATIC_MODEL, label: t('settings.transcription.models.automatic') },
          ...choices,
        ].map((choice) => (
          <div key={choice.value} className="flex items-center gap-2">
            <RadioGroupItem id={`${id}-${choice.value}`} value={choice.value} />
            <Label htmlFor={`${id}-${choice.value}`} className="text-text">
              {choice.label}
            </Label>
          </div>
        ))}
      </RadioGroup>
      {!hasInstalledModel(status) && (
        <p className="text-2xs text-warning">{t('settings.transcription.models.none')}</p>
      )}
    </div>
  );
}
