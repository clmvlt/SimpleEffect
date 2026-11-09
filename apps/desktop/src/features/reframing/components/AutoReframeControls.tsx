import type { ItemEntity, TrackingSettings } from '@se/protocol';
import { Button, Icon, Progress } from '@se/ui';
import { ScanFace } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useAssetJob } from '../../media-bin';
import { useAutoReframe } from '../hooks/use-auto-reframe';
import { useVisionStatus } from '../hooks/use-vision';
import { shownTracking, trackingOf } from '../services/tracking';
import { TrackingSettingsFields } from './TrackingSettingsFields';

/** Props of {@link AutoReframeControls}. */
export interface AutoReframeControlsProps {
  /** The selected media clips of video tracks. */
  readonly clips: readonly ItemEntity[];
}

/**
 * « Suivi du sujet » of the inspector's « Cadrage » (prompt 26): « Recadrer automatiquement » (or
 * « Relancer » for clips already tracked), the face analysis's progress, and the tracking
 * settings — applied at once to tracked clips (their faces are already analysed), used by the
 * next reframing otherwise. Key it by the selection so its draft follows the clips.
 */
export function AutoReframeControls({ clips }: AutoReframeControlsProps) {
  const { t } = useTranslation();
  const status = useVisionStatus();
  const auto = useAutoReframe();
  const [draft, setDraft] = useState<TrackingSettings | null>(null);
  const tracked = clips.length > 0 && clips.every((clip) => trackingOf(clip) !== null);
  const settings = draft ?? shownTracking(clips);
  const [first] = clips;
  const assetId = first?.kind.kind === 'mediaClip' ? first.kind.assetId : null;
  const job = useAssetJob(assetId, 'subjectDetection');
  if (status?.available === false) {
    return <p className="px-3 pb-2 text-xs text-text-muted">{t('editor.reframing.unavailable')}</p>;
  }
  const ready = auto !== null && status !== null && !auto.busy;
  const apply = (next: TrackingSettings) => {
    if (ready)
      void auto.reframe(
        clips.map((clip) => clip.id),
        next,
      );
  };
  return (
    <div className="flex flex-col gap-1 pt-1">
      <div className="flex flex-col gap-1.5 px-3">
        <Button
          size="xs"
          variant="secondary"
          disabled={!ready || job?.kind === 'running' || job?.kind === 'queued'}
          onClick={() => {
            apply(settings);
          }}
        >
          <Icon icon={ScanFace} size="sm" />
          {t(tracked ? 'editor.reframing.relaunch' : 'editor.reframing.start')}
        </Button>
        {job?.kind === 'running' || job?.kind === 'queued' ? (
          <div className="flex flex-col gap-1">
            <span className="text-xs text-text-muted">
              {job.kind === 'running' && job.progress !== null
                ? t('editor.reframing.progress', { percent: Math.round(job.progress * 100) })
                : t('editor.reframing.progressUnknown')}
            </span>
            <Progress
              size="sm"
              value={job.kind === 'running' && job.progress !== null ? job.progress * 100 : null}
              aria-label={t('editor.reframing.progressLabel')}
            />
          </div>
        ) : null}
        {job?.kind === 'failed' ? (
          <p className="text-xs text-danger">
            {t('editor.reframing.analysisFailed', { message: job.message })}
          </p>
        ) : null}
        {tracked ? (
          <p className="text-xs text-text-muted">{t('editor.reframing.trackedHint')}</p>
        ) : null}
      </div>
      <TrackingSettingsFields
        settings={settings}
        disabled={!ready}
        onChange={setDraft}
        onCommit={tracked ? apply : undefined}
      />
    </div>
  );
}
