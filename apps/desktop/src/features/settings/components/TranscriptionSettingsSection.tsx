import type { TranscriptionStatus } from '@se/protocol';
import { Field, Separator, Switch } from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { TranscriptionActions } from '../hooks/use-transcription-settings';
import { GpuRuntimeRow } from './GpuRuntimeRow';
import { TranscriptionModelsList } from './TranscriptionModelsList';
import { TranscriptionOptions } from './TranscriptionOptions';

/** Props of {@link TranscriptionSettingsSection}. */
export interface TranscriptionSettingsSectionProps {
  readonly status: TranscriptionStatus;
  readonly actions: TranscriptionActions;
}

/**
 * « Transcription » (prompt 21): whisper.cpp found or why not, where transcriptions compute (GPU,
 * the GPU pack), the models, and the options. Nothing is downloaded without the user's consent.
 */
export function TranscriptionSettingsSection({
  status,
  actions,
}: TranscriptionSettingsSectionProps) {
  return (
    <div className="flex flex-col gap-4">
      <ProgramStatus status={status} />
      <Separator />
      <ComputeSection status={status} actions={actions} />
      <Separator />
      <TranscriptionModelsList status={status} actions={actions} />
      <Separator />
      <TranscriptionOptions status={status} actions={actions} />
    </div>
  );
}

/** « Programme »: the `whisper-cli` used, or what is wrong. */
function ProgramStatus({ status }: { readonly status: TranscriptionStatus }) {
  const { t } = useTranslation();
  const { path, problem } = status.whisper;
  return (
    <section aria-labelledby="settings-transcription-program" className="flex flex-col gap-2">
      <h3 id="settings-transcription-program" className="text-sm font-semibold text-text">
        {t('settings.transcription.program.title')}
      </h3>
      {path !== null && (
        <p className="truncate text-xs text-text-muted select-text" title={path}>
          {t('settings.transcription.program.found', { path })}
        </p>
      )}
      {(problem !== null || path === null) && (
        <p className="text-xs break-words text-warning select-text" role="alert">
          {problem ?? t('settings.transcription.program.missing')}
        </p>
      )}
    </section>
  );
}

/** « Calcul »: the GPU detected, where the next and last transcriptions run, « Utiliser le GPU ». */
function ComputeSection({ status, actions }: TranscriptionSettingsSectionProps) {
  const { t } = useTranslation();
  return (
    <section aria-labelledby="settings-transcription-compute" className="flex flex-col gap-3">
      <h3 id="settings-transcription-compute" className="text-sm font-semibold text-text">
        {t('settings.transcription.compute.title')}
      </h3>
      <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 text-xs">
        <dt className="text-text-muted">{t('settings.transcription.compute.gpu')}</dt>
        <dd className="text-text">{status.gpuName ?? t('settings.transcription.compute.noGpu')}</dd>
        <dt className="text-text-muted">{t('settings.transcription.compute.nextRun')}</dt>
        <dd className="text-text">{t(`settings.transcription.backends.${status.backend}`)}</dd>
        {status.lastBackend !== null && (
          <>
            <dt className="text-text-muted">{t('settings.transcription.compute.lastRun')}</dt>
            <dd className="text-text">
              {t(`settings.transcription.backends.${status.lastBackend}`)}
            </dd>
          </>
        )}
      </dl>
      <Field
        label={t('settings.transcription.compute.useGpu')}
        description={t('settings.transcription.compute.useGpuDescription')}
        orientation="horizontal"
      >
        <Switch
          checked={status.preferences.useGpu}
          onCheckedChange={(checked) => {
            actions.save({ useGpu: checked });
          }}
        />
      </Field>
      {status.gpuRuntime !== null && (
        <GpuRuntimeRow runtime={status.gpuRuntime} actions={actions} />
      )}
    </section>
  );
}
