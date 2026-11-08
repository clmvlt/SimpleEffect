import type { DownloadTarget, GpuRuntimeInfo } from '@se/protocol';
import { Badge } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { byteSize } from '../../../shared/text';
import type { TranscriptionActions } from '../hooks/use-transcription-settings';
import { downloadPhase } from '../services/transcription-downloads';
import { ConfirmButton } from './ConfirmButton';
import { DownloadProgressRow } from './DownloadProgressRow';

const GPU_RUNTIME: DownloadTarget = { kind: 'gpuRuntime' };

/** Props of {@link GpuRuntimeRow}. */
export interface GpuRuntimeRowProps {
  readonly runtime: GpuRuntimeInfo;
  readonly actions: TranscriptionActions;
}

/**
 * The GPU pack (Windows: the official CUDA build of whisper.cpp): its size and « Installer le pack
 * GPU » behind a consent dialog, its download, or « Installé » with « Supprimer ».
 */
export function GpuRuntimeRow({ runtime, actions }: GpuRuntimeRowProps) {
  const { t, i18n } = useTranslation();
  const values = { name: runtime.name, size: byteSize(runtime.downloadSize, i18n.language) };
  const phase = downloadPhase(runtime);
  return (
    <section aria-labelledby="settings-gpu-pack" className="flex flex-col gap-2">
      <h4 id="settings-gpu-pack" className="text-xs font-medium text-text">
        {t('settings.transcription.gpuPack.title')}
      </h4>
      <p className="text-2xs text-text-muted">
        {t('settings.transcription.gpuPack.description', values)}
      </p>
      <div className="flex min-h-control-xs items-center gap-2">
        {runtime.download !== null && (
          <DownloadProgressRow
            progress={runtime.download}
            name={t('settings.transcription.gpuPack.name')}
            onCancel={() => {
              actions.cancelDownload(GPU_RUNTIME);
            }}
            onRetry={() => {
              actions.startDownload(GPU_RUNTIME);
            }}
          />
        )}
        {phase === 'missing' && (
          <ConfirmButton
            variant="primary"
            title={t('settings.transcription.gpuPack.confirmTitle')}
            description={
              <>
                <p>{t('settings.transcription.gpuPack.confirmDescription', values)}</p>
                <p>{t('settings.transcription.localOnly')}</p>
              </>
            }
            confirmLabel={t('settings.transcription.download')}
            cancelLabel={t('settings.transcription.cancel')}
            onConfirm={() => {
              actions.startDownload(GPU_RUNTIME);
            }}
          >
            {t('settings.transcription.gpuPack.install', values)}
          </ConfirmButton>
        )}
        {phase === 'installed' && (
          <>
            <Badge variant="success" size="xs">
              {t('settings.transcription.gpuPack.installed')}
            </Badge>
            <ConfirmButton
              variant="ghost"
              title={t('settings.transcription.gpuPack.removeTitle')}
              description={t('settings.transcription.gpuPack.removeDescription')}
              confirmLabel={t('settings.transcription.remove')}
              confirmVariant="danger"
              cancelLabel={t('settings.transcription.cancel')}
              onConfirm={() => {
                actions.removeFile(GPU_RUNTIME);
              }}
            >
              {t('settings.transcription.remove')}
            </ConfirmButton>
          </>
        )}
      </div>
    </section>
  );
}
