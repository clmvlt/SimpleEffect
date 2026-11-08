import type { DownloadTarget, TranscriptionStatus } from '@se/protocol';
import { Button, Field, Switch } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useDesktopBridge } from '../../../shared/desktop';
import { byteSize } from '../../../shared/text';
import type { TranscriptionActions } from '../hooks/use-transcription-settings';
import { ConfirmButton } from './ConfirmButton';
import { DownloadProgressRow } from './DownloadProgressRow';

const VOICE_MODEL: DownloadTarget = { kind: 'voiceModel' };

/** Props of {@link TranscriptionOptions}. */
export interface TranscriptionOptionsProps {
  readonly status: TranscriptionStatus;
  readonly actions: TranscriptionActions;
}

/**
 * « Options »: voice detection (with the voice model's state), transcription at import, a custom
 * whisper.cpp folder and the models folder.
 */
export function TranscriptionOptions({ status, actions }: TranscriptionOptionsProps) {
  const { t } = useTranslation();
  const { preferences } = status;
  return (
    <section aria-labelledby="settings-transcription-options" className="flex flex-col gap-4">
      <h3 id="settings-transcription-options" className="text-sm font-semibold text-text">
        {t('settings.transcription.options.title')}
      </h3>
      <div className="flex flex-col gap-2">
        <Field
          label={t('settings.transcription.options.voiceDetection')}
          description={t('settings.transcription.options.voiceDetectionDescription')}
          orientation="horizontal"
        >
          <Switch
            checked={preferences.voiceDetection}
            onCheckedChange={(checked) => {
              actions.save({ voiceDetection: checked });
            }}
          />
        </Field>
        {!status.voiceModel.installed && <VoiceModelState status={status} actions={actions} />}
      </div>
      <Field
        label={t('settings.transcription.options.autoTranscribe')}
        description={t('settings.transcription.options.autoTranscribeDescription')}
        orientation="horizontal"
      >
        <Switch
          checked={preferences.autoTranscribe}
          onCheckedChange={(checked) => {
            actions.save({ autoTranscribe: checked });
          }}
        />
      </Field>
      <Field
        label={t('settings.transcription.options.keepHesitations')}
        description={t('settings.transcription.options.keepHesitationsDescription')}
        orientation="horizontal"
      >
        <Switch
          checked={preferences.keepHesitations}
          onCheckedChange={(checked) => {
            actions.save({ keepHesitations: checked });
          }}
        />
      </Field>
      <WhisperFolder status={status} actions={actions} />
      <div className="flex flex-col gap-1.5">
        <span className="text-xs font-medium text-text">
          {t('settings.transcription.options.modelsDir')}
        </span>
        <div className="flex items-center gap-2">
          <span
            className="min-w-0 flex-1 truncate text-xs text-text-muted select-text"
            title={status.modelsDir}
          >
            {status.modelsDir}
          </span>
          <Button size="xs" variant="secondary" onClick={actions.revealModelsDir}>
            {t('settings.transcription.options.showModelsDir')}
          </Button>
        </div>
      </div>
    </section>
  );
}

/** « Modèle de voix manquant » with its download (after consent), or its running download. */
function VoiceModelState({ status, actions }: TranscriptionOptionsProps) {
  const { t, i18n } = useTranslation();
  const { voiceModel } = status;
  const size = byteSize(voiceModel.size, i18n.language);
  if (voiceModel.download !== null) {
    return (
      <div className="flex items-center gap-2">
        <DownloadProgressRow
          progress={voiceModel.download}
          name={t('settings.transcription.voiceModel.name')}
          onCancel={() => {
            actions.cancelDownload(VOICE_MODEL);
          }}
          onRetry={() => {
            actions.startDownload(VOICE_MODEL);
          }}
        />
      </div>
    );
  }
  return (
    <div className="flex items-center gap-2">
      <span className="flex-1 text-2xs text-warning">
        {t('settings.transcription.voiceModel.missing')}
      </span>
      <ConfirmButton
        title={t('settings.transcription.voiceModel.confirmTitle')}
        description={
          <>
            <p>{t('settings.transcription.voiceModel.confirmDescription', { size })}</p>
            <p>{t('settings.transcription.localOnly')}</p>
          </>
        }
        confirmLabel={t('settings.transcription.download')}
        cancelLabel={t('settings.transcription.cancel')}
        onConfirm={() => {
          actions.startDownload(VOICE_MODEL);
        }}
      >
        {t('settings.transcription.voiceModel.download', { size })}
      </ConfirmButton>
    </div>
  );
}

/** The folder of the user's own `whisper-cli` build (« Choisir… »), or « Automatique ». */
function WhisperFolder({ status, actions }: TranscriptionOptionsProps) {
  const { t } = useTranslation();
  const bridge = useDesktopBridge();
  const { whisperDir } = status.preferences;
  const choose = async () => {
    const folder = await bridge.chooseFolder({
      title: t('settings.transcription.options.chooseWhisperDirTitle'),
      buttonLabel: t('settings.transcription.options.chooseWhisperDirButton'),
      ...(whisperDir === null ? {} : { defaultPath: whisperDir }),
    });
    if (folder !== null) actions.save({ whisperDir: folder });
  };
  return (
    <div className="flex flex-col gap-1.5">
      <span className="text-xs font-medium text-text">
        {t('settings.transcription.options.whisperDir')}
      </span>
      <div className="flex items-center gap-2">
        <span
          className="min-w-0 flex-1 truncate text-xs text-text-muted select-text"
          title={whisperDir ?? undefined}
        >
          {whisperDir ?? t('settings.transcription.options.whisperDirDefault')}
        </span>
        <Button size="xs" variant="secondary" onClick={() => void choose()}>
          {t('settings.transcription.options.chooseWhisperDir')}
        </Button>
        {whisperDir !== null && (
          <Button
            size="xs"
            variant="ghost"
            onClick={() => {
              actions.save({ whisperDir: null });
            }}
          >
            {t('settings.transcription.options.resetWhisperDir')}
          </Button>
        )}
      </div>
    </div>
  );
}
