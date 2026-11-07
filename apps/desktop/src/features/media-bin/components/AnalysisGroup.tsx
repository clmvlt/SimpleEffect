import type { Asset } from '@se/protocol';
import { PropertyGroup, PropertyRow } from '@se/ui';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { audioFigures } from '../services/analysis-format';
import { assetJobStatus } from '../services/job-status';
import { jobsStore } from '../store/jobs-store';
import { InfoValue } from './InfoValue';

/** Props of {@link AnalysisGroup}. */
export interface AnalysisGroupProps {
  readonly asset: Asset;
}

/**
 * What the analyses measured: integrated loudness, true peak, loudness range and silences of the
 * sound, number of shot changes of a video. « Analyse en cours… » until the summaries arrive.
 */
export function AnalysisGroup({ asset }: AnalysisGroupProps) {
  const { t, i18n } = useTranslation();
  const jobs = useStore(jobsStore, (state) => state.jobs);
  const status = assetJobStatus(jobs, asset.id);
  const analysing = [...status.running.map((job) => job.kind), ...status.queued].some(
    (kind) => kind === 'audioAnalysis' || kind === 'sceneDetection',
  );
  const hasSound = asset.kind !== 'image' && (asset.mediaInfo?.audioStreams.length ?? 0) > 0;
  const hasPicture = asset.kind === 'video';
  if (!hasSound && !hasPicture) {
    return null;
  }
  const { audio, scenes } = asset.analysis;
  const pending = t('editor.media.info.analysis.pending');
  const notMeasured = t('editor.media.info.analysis.notMeasured');
  const waiting = analysing ? pending : notMeasured;
  const figures = audio === null ? null : audioFigures(audio, i18n.language);
  const measured = (value: string | null, key: 'loudnessValue' | 'truePeakValue' | 'rangeValue') =>
    value === null
      ? t('editor.media.info.analysis.silent')
      : t(`editor.media.info.analysis.${key}`, { value });
  return (
    <PropertyGroup title={t('editor.media.info.analysis.title')}>
      {hasSound && (
        <>
          <PropertyRow label={t('editor.media.info.analysis.loudness')}>
            <InfoValue>
              {figures === null ? waiting : measured(figures.loudness, 'loudnessValue')}
            </InfoValue>
          </PropertyRow>
          <PropertyRow label={t('editor.media.info.analysis.truePeak')}>
            <InfoValue>
              {figures === null ? waiting : measured(figures.truePeak, 'truePeakValue')}
            </InfoValue>
          </PropertyRow>
          <PropertyRow label={t('editor.media.info.analysis.range')}>
            <InfoValue>
              {figures === null
                ? waiting
                : figures.loudnessRange === null
                  ? notMeasured
                  : t('editor.media.info.analysis.rangeValue', { value: figures.loudnessRange })}
            </InfoValue>
          </PropertyRow>
          <PropertyRow label={t('editor.media.info.analysis.silences')}>
            <InfoValue>
              {figures === null
                ? waiting
                : t('editor.media.info.analysis.silencesValue', {
                    count: figures.silenceCount,
                    seconds: figures.silenceSeconds,
                  })}
            </InfoValue>
          </PropertyRow>
        </>
      )}
      {hasPicture && (
        <PropertyRow label={t('editor.media.info.analysis.scenes')}>
          <InfoValue>
            {scenes === null
              ? waiting
              : t('editor.media.info.analysis.scenesValue', { count: scenes.changeCount })}
          </InfoValue>
        </PropertyRow>
      )}
    </PropertyGroup>
  );
}
