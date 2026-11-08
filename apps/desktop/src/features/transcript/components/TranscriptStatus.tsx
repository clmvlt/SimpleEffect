import type { Asset, AssetId } from '@se/protocol';
import { Button, Progress } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useTranscriptionJob } from '../../media-bin';

/** Props of {@link UntranscribedMedia}. */
export interface UntranscribedMediaProps {
  readonly assets: readonly Asset[];
  readonly onTranscribe: (assetIds: readonly AssetId[]) => void;
}

/** The file name of an asset. */
function assetName(asset: Asset): string {
  return asset.originalPath.split(/[\\/]/).pop() ?? asset.originalPath;
}

/**
 * The media in view without a transcript: each with « Transcrire » or its transcription's
 * progress, and « Tout transcrire » when there are several.
 */
export function UntranscribedMedia({ assets, onTranscribe }: UntranscribedMediaProps) {
  const { t } = useTranslation();
  if (assets.length === 0) return null;
  return (
    <section
      aria-label={t('editor.transcript.untranscribedLabel')}
      className="flex shrink-0 flex-col gap-1.5 border-b border-border p-2"
    >
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs text-text-muted">
          {t('editor.transcript.untranscribed', { count: assets.length })}
        </span>
        {assets.length > 1 && (
          <Button
            size="xs"
            onClick={() => {
              onTranscribe(assets.map((asset) => asset.id));
            }}
          >
            {t('editor.transcript.transcribeAll')}
          </Button>
        )}
      </div>
      {assets.map((asset) => (
        <UntranscribedRow key={asset.id} asset={asset} onTranscribe={onTranscribe} />
      ))}
    </section>
  );
}

function UntranscribedRow({
  asset,
  onTranscribe,
}: {
  readonly asset: Asset;
  readonly onTranscribe: (assetIds: readonly AssetId[]) => void;
}) {
  const { t } = useTranslation();
  const job = useTranscriptionJob(asset.id);
  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="min-w-0 flex-1 truncate" title={assetName(asset)}>
        {assetName(asset)}
      </span>
      {job?.kind === 'running' || job?.kind === 'queued' ? (
        <div className="flex w-28 items-center gap-1.5">
          <Progress
            className="flex-1"
            value={job.kind === 'running' ? Math.round((job.progress ?? 0) * 100) : 0}
            aria-label={t('editor.transcript.progressLabel', { name: assetName(asset) })}
          />
          <span className="text-2xs text-text-muted tabular-nums">
            {job.kind === 'running'
              ? t('editor.transcript.percent', { percent: Math.round((job.progress ?? 0) * 100) })
              : t('editor.transcript.queued')}
          </span>
        </div>
      ) : (
        <Button
          size="xs"
          variant={job?.kind === 'failed' ? 'secondary' : 'primary'}
          title={job?.kind === 'failed' ? job.message : undefined}
          onClick={() => {
            onTranscribe([asset.id]);
          }}
        >
          {job?.kind === 'failed'
            ? t('editor.transcript.retry')
            : t('editor.transcript.transcribe')}
        </Button>
      )}
    </div>
  );
}
