import type { DownloadProgress } from '@se/protocol';
import { Button, Progress, Spinner } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { byteSize } from '../../../shared/text';
import { downloadPercent } from '../services/transcription-downloads';

/** Props of {@link DownloadProgressRow}. */
export interface DownloadProgressRowProps {
  readonly progress: DownloadProgress;
  /** What is downloaded (`large-v3-turbo`, « pack GPU »), for the progress bar's name. */
  readonly name: string;
  readonly onCancel: () => void;
  /** Starts the download again after a failure (the user already consented to it). */
  readonly onRetry: () => void;
}

/**
 * A download as it runs: a progress bar with the bytes received and « Annuler », « Vérification… »
 * while its SHA-256 is checked, or why it failed with « Réessayer ».
 */
export function DownloadProgressRow({
  progress,
  name,
  onCancel,
  onRetry,
}: DownloadProgressRowProps) {
  const { t, i18n } = useTranslation();
  switch (progress.kind) {
    case 'downloading':
      return (
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <Progress
            value={downloadPercent(progress)}
            aria-label={t('settings.transcription.downloadProgress', { name })}
            className="min-w-16 flex-1"
          />
          <span className="shrink-0 text-2xs text-text-muted tabular-nums">
            {t('settings.transcription.downloadAmount', {
              received: byteSize(progress.received, i18n.language),
              total: byteSize(progress.total, i18n.language),
            })}
          </span>
          <Button size="xs" variant="ghost" onClick={onCancel}>
            {t('settings.transcription.cancelDownload')}
          </Button>
        </div>
      );
    case 'verifying':
      return (
        <span className="flex items-center gap-1.5 text-xs text-text-muted" role="status">
          <Spinner size="sm" label={t('settings.transcription.verifying')} />
          {t('settings.transcription.verifying')}
        </span>
      );
    case 'failed':
      return (
        <div className="flex min-w-0 flex-1 items-center gap-2">
          <span className="min-w-0 flex-1 text-xs break-words text-danger select-text" role="alert">
            {progress.message}
          </span>
          <Button size="xs" variant="secondary" onClick={onRetry}>
            {t('settings.transcription.retry')}
          </Button>
        </div>
      );
  }
}
