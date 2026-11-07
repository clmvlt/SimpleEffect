import { playbackSource, type PlaybackSource } from '@se/engine/sources';
import type { Asset, DerivedKind } from '@se/protocol';
import { Button, Progress, PropertyGroup, PropertyRow } from '@se/ui';
import type { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { derivedDisplay, type DerivedDisplay } from '../services/derived-display';
import { assetJobStatus } from '../services/job-status';
import { jobsStore } from '../store/jobs-store';
import { InfoValue } from './InfoValue';

/** Props of {@link DerivedMediaGroup}. */
export interface DerivedMediaGroupProps {
  readonly asset: Asset;
  readonly onRetry: (asset: Asset, derived: DerivedKind) => void;
}

function sourceLabel(t: TFunction, source: PlaybackSource): string {
  switch (source.kind) {
    case 'original':
      return t('editor.media.info.derived.sources.original');
    case 'mezzanine':
      return t('editor.media.info.derived.sources.mezzanine');
    case 'preparing':
      return t('editor.media.info.derived.sources.preparing');
    case 'unavailable':
      return source.reason === 'missing'
        ? t('editor.media.info.derived.sources.missing')
        : t('editor.media.info.derived.sources.failed');
  }
}

function stateLabel(t: TFunction, kind: DerivedKind, display: DerivedDisplay): string {
  switch (display.kind) {
    case 'none':
      return t('editor.media.info.derived.none');
    case 'queued':
      return t('editor.media.info.derived.queued');
    case 'running':
      return display.progress === null
        ? t('editor.media.info.derived.runningUnknown')
        : t('editor.media.info.derived.running', { percent: Math.round(display.progress * 100) });
    case 'ready':
      return kind === 'mezzanine'
        ? t('editor.media.info.derived.readyMezzanine')
        : t('editor.media.info.derived.readyProxy');
    case 'failed':
      return t('editor.media.info.derived.failed', { message: display.message });
  }
}

/**
 * How the asset plays (original, mezzanine, still in preparation) and the state of its mezzanine
 * and proxy: waiting, progress, ready, or the failure's reason with « Réessayer ».
 */
export function DerivedMediaGroup({ asset, onRetry }: DerivedMediaGroupProps) {
  const { t } = useTranslation();
  const jobs = useStore(jobsStore, (state) => state.jobs);
  const status = assetJobStatus(jobs, asset.id);
  const kinds: readonly DerivedKind[] =
    asset.kind === 'video' ? ['mezzanine', 'proxy'] : ['mezzanine'];
  return (
    <PropertyGroup title={t('editor.media.info.derived.title')}>
      <PropertyRow label={t('editor.media.info.derived.playback')}>
        <InfoValue>{sourceLabel(t, playbackSource(asset))}</InfoValue>
      </PropertyRow>
      {kinds.map((kind) => {
        const display = derivedDisplay(asset, kind, status);
        const label = stateLabel(t, kind, display);
        return (
          <PropertyRow key={kind} label={t(`editor.media.info.derived.${kind}`)}>
            <span className="flex min-w-0 flex-1 flex-col gap-1">
              <span className="flex min-w-0 items-center gap-2">
                <span
                  className={
                    display.kind === 'failed' ? 'min-w-0 flex-1 text-warning' : 'min-w-0 flex-1'
                  }
                >
                  <InfoValue title={label}>{label}</InfoValue>
                </span>
                {display.kind === 'failed' && (
                  <Button
                    size="xs"
                    variant="secondary"
                    onClick={() => {
                      onRetry(asset, kind);
                    }}
                  >
                    {t('editor.media.info.derived.retry')}
                  </Button>
                )}
              </span>
              {(display.kind === 'running' || display.kind === 'queued') && (
                <Progress
                  size="xs"
                  value={
                    display.kind === 'running' && display.progress !== null
                      ? display.progress * 100
                      : null
                  }
                  aria-label={t(`editor.media.info.derived.${kind}`)}
                  aria-valuetext={label}
                />
              )}
            </span>
          </PropertyRow>
        );
      })}
    </PropertyGroup>
  );
}
