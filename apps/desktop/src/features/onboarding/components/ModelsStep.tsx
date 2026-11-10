import { Spinner } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { TranscriptionModelsList, useTranscriptionSettings } from '../../settings';

/** Props of {@link ModelsStep}. */
export interface ModelsStepProps {
  readonly active: boolean;
}

/**
 * « Modèles (facultatif) »: the transcription models of « Réglages → Transcription », each
 * download behind its consent (size, source, SHA-256).
 */
export function ModelsStep({ active }: ModelsStepProps) {
  const { t } = useTranslation();
  const transcription = useTranscriptionSettings(active);
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm text-text-muted">{t('onboarding.models.intro')}</p>
      {transcription.status === null ? (
        <div className="flex justify-center py-6">
          <Spinner size="lg" label={t('onboarding.models.loading')} />
        </div>
      ) : (
        <div className="max-h-[40vh] overflow-auto">
          <TranscriptionModelsList status={transcription.status} actions={transcription} />
        </div>
      )}
      <p className="text-2xs text-text-muted">{t('onboarding.models.later')}</p>
    </div>
  );
}
