import type { DownloadTarget, TranscriptionModelInfo } from '@se/protocol';
import { Badge } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { byteSize } from '../../../shared/text';
import type { TranscriptionActions } from '../hooks/use-transcription-settings';
import { downloadPhase } from '../services/transcription-downloads';
import { modelLabel } from '../services/transcription-models';
import { ConfirmButton } from './ConfirmButton';
import { DownloadProgressRow } from './DownloadProgressRow';

/** Props of {@link TranscriptionModelRow}. */
export interface TranscriptionModelRowProps {
  readonly model: TranscriptionModelInfo;
  /** Size of the voice model when a download of this model also brings it, else `null`. */
  readonly voiceModelSize: number | null;
  readonly actions: TranscriptionActions;
}

/**
 * One model: its name (« Importé » and « Recommandé » tags), its size, and « Télécharger » behind a
 * consent dialog, its download, or « Supprimer » (after confirmation) once installed.
 */
export function TranscriptionModelRow({
  model,
  voiceModelSize,
  actions,
}: TranscriptionModelRowProps) {
  const { t, i18n } = useTranslation();
  const name = modelLabel(model);
  const values = { name, size: byteSize(model.size, i18n.language) };
  const target: DownloadTarget = { kind: 'model', modelId: model.id };
  const phase = downloadPhase(model);
  return (
    <li aria-label={name} className="flex min-h-9 items-center gap-2 py-1.5">
      <span className="min-w-0 truncate text-xs font-medium text-text select-text" title={name}>
        {name}
      </span>
      {model.custom && <Badge size="xs">{t('settings.transcription.models.importedTag')}</Badge>}
      {model.recommended && (
        <Badge variant="accent" size="xs">
          {t('settings.transcription.models.recommended')}
        </Badge>
      )}
      <span className="shrink-0 text-2xs text-text-muted tabular-nums">{values.size}</span>
      <div className="ml-auto flex min-w-0 items-center justify-end gap-2">
        {model.download !== null && (
          <DownloadProgressRow
            progress={model.download}
            name={name}
            onCancel={() => {
              actions.cancelDownload(target);
            }}
            onRetry={() => {
              actions.startDownload(target);
            }}
          />
        )}
        {phase === 'missing' && (
          <ConfirmButton
            title={t('settings.transcription.models.confirmTitle', values)}
            description={
              <>
                <p>{t('settings.transcription.models.confirmDescription', values)}</p>
                {voiceModelSize !== null && (
                  <p>
                    {t('settings.transcription.models.voiceModelToo', {
                      size: byteSize(voiceModelSize, i18n.language),
                    })}
                  </p>
                )}
                <p>{t('settings.transcription.localOnly')}</p>
              </>
            }
            confirmLabel={t('settings.transcription.download')}
            cancelLabel={t('settings.transcription.cancel')}
            onConfirm={() => {
              actions.startDownload(target);
            }}
          >
            {t('settings.transcription.download')}
          </ConfirmButton>
        )}
        {phase === 'installed' && (
          <ConfirmButton
            variant="ghost"
            title={t('settings.transcription.models.removeTitle', values)}
            description={t(
              model.custom
                ? 'settings.transcription.models.removeImportedDescription'
                : 'settings.transcription.models.removeDescription',
              values,
            )}
            confirmLabel={t('settings.transcription.remove')}
            confirmVariant="danger"
            cancelLabel={t('settings.transcription.cancel')}
            onConfirm={() => {
              actions.removeFile(target);
            }}
          >
            {t('settings.transcription.remove')}
          </ConfirmButton>
        )}
      </div>
    </li>
  );
}
