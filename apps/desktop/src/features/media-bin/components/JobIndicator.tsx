import type { Asset } from '@se/protocol';
import { Icon, Progress, Tooltip } from '@se/ui';
import { TriangleAlert } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { assetName } from '../services/asset-list';
import { assetJobStatus } from '../services/job-status';
import { jobsStore } from '../store/jobs-store';

/** Props of {@link JobIndicator}. */
export interface JobIndicatorProps {
  readonly asset: Asset;
}

/**
 * The background work on an asset: a progress bar while thumbnails or peaks are generated (or
 * wait for a slot), a warning when the latest attempt failed. Nothing when idle.
 */
export function JobIndicator({ asset }: JobIndicatorProps) {
  const { t } = useTranslation();
  const jobs = useStore(jobsStore, (state) => state.jobs);
  const status = assetJobStatus(jobs, asset.id);
  const task = (kind: 'thumbnails' | 'waveformPeaks') => t(`editor.media.jobs.${kind}`);
  const [running] = status.running;
  if (running !== undefined || status.queued.length > 0) {
    const progress = running?.progress ?? null;
    const label =
      running === undefined
        ? t('editor.media.jobs.queued')
        : progress === null
          ? t('editor.media.jobs.runningUnknown', { task: task(running.kind) })
          : t('editor.media.jobs.running', {
              task: task(running.kind),
              percent: Math.round(progress * 100),
            });
    return (
      <Tooltip content={label}>
        <Progress
          size="xs"
          value={progress === null ? null : progress * 100}
          aria-label={t('editor.media.jobs.progressLabel', { name: assetName(asset) })}
          aria-valuetext={label}
        />
      </Tooltip>
    );
  }
  const [failed] = status.failed;
  if (failed === undefined) {
    return null;
  }
  const message = t('editor.media.jobs.failed', {
    task: task(failed.kind),
    message: failed.message,
  });
  return (
    <Tooltip content={message}>
      <span role="img" aria-label={message} className="inline-flex text-warning">
        <Icon icon={TriangleAlert} size="sm" />
      </span>
    </Tooltip>
  );
}
