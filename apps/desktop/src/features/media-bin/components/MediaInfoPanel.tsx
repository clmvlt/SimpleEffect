import type { Asset, AudioStreamInfo, DerivedKind, StreamDecodability } from '@se/protocol';
import { Button, IconButton, PropertyGroup, PropertyRow } from '@se/ui';
import { X } from '@se/ui/icons';
import type { TFunction } from 'i18next';
import { useTranslation } from 'react-i18next';

import { formatDuration } from '../../../shared/time';
import { codecLabel, displaySize, framesPerSecond, kilohertz } from '../services/asset-details';
import { assetName } from '../services/asset-list';
import { AnalysisGroup } from './AnalysisGroup';
import { DerivedMediaGroup } from './DerivedMediaGroup';
import { InfoValue as Value } from './InfoValue';

/** Props of {@link MediaInfoPanel}. */
export interface MediaInfoPanelProps {
  readonly asset: Asset;
  readonly onClose: () => void;
  readonly onRelink: (asset: Asset) => void;
  readonly onRetryDerived: (asset: Asset, derived: DerivedKind) => void;
}

function decodability(t: TFunction, state: StreamDecodability): string {
  return t(`editor.media.info.decodable.${state}`);
}

function audioSummary(t: TFunction, stream: AudioStreamInfo): string {
  return [
    codecLabel(stream.codec, stream.profile),
    t('editor.media.info.channels', { count: stream.channels }),
    t('editor.media.info.sampleRate', { rate: kilohertz(stream) }),
  ].join(' · ');
}

/**
 * Details of the selected asset: file, streams, decodability, how it plays (mezzanine, proxy),
 * its analyses, and « Relier… » when missing.
 */
export function MediaInfoPanel({ asset, onClose, onRelink, onRetryDerived }: MediaInfoPanelProps) {
  const { t } = useTranslation();
  const info = asset.mediaInfo;
  return (
    <aside
      aria-label={t('editor.media.info.title')}
      className="flex max-h-[50%] shrink-0 flex-col overflow-auto border-t border-border bg-panel-raised"
    >
      <div className="flex items-center gap-2 px-2 pt-2">
        <span className="truncate text-xs font-semibold text-text">{assetName(asset)}</span>
        <IconButton
          icon={X}
          size="xs"
          label={t('editor.media.info.close')}
          className="ml-auto"
          onClick={onClose}
        />
      </div>
      {asset.missing && (
        <div className="flex items-center gap-2 px-2 pt-2">
          <span className="min-w-0 flex-1 truncate text-xs text-warning" title={asset.originalPath}>
            {t('editor.media.info.missingPath', { path: asset.originalPath })}
          </span>
          <Button
            size="xs"
            variant="primary"
            onClick={() => {
              onRelink(asset);
            }}
          >
            {t('editor.media.menu.relink')}
          </Button>
        </div>
      )}
      <PropertyGroup title={t('editor.media.info.file')}>
        <PropertyRow label={t('editor.media.info.path')}>
          <Value title={asset.originalPath}>{asset.originalPath}</Value>
        </PropertyRow>
        <PropertyRow label={t('editor.media.info.kind')}>
          <Value>{t(`editor.media.kinds.${asset.kind}`)}</Value>
        </PropertyRow>
        {info !== null && info.duration !== null && (
          <PropertyRow label={t('editor.media.info.duration')}>
            <Value>{formatDuration(info.duration)}</Value>
          </PropertyRow>
        )}
        <PropertyRow label={t('editor.media.info.container')}>
          <Value>{info?.container ?? t('editor.media.info.unknown')}</Value>
        </PropertyRow>
      </PropertyGroup>
      <VideoDetails asset={asset} t={t} />
      {info !== null && info.audioStreams.length > 0 && (
        <PropertyGroup title={t('editor.media.info.audio')}>
          {info.audioStreams.map((stream, index) => (
            <PropertyRow
              key={index}
              label={t('editor.media.info.audioStream', { index: index + 1 })}
            >
              <Value>{audioSummary(t, stream)}</Value>
            </PropertyRow>
          ))}
          <PropertyRow label={t('editor.media.info.decodability')}>
            <Value>{decodability(t, asset.decodability.audio)}</Value>
          </PropertyRow>
        </PropertyGroup>
      )}
      <DerivedMediaGroup asset={asset} onRetry={onRetryDerived} />
      <AnalysisGroup asset={asset} />
    </aside>
  );
}

/** The picture stream's rows (nothing for sound files). */
function VideoDetails({ asset, t }: { asset: Asset; t: TFunction }) {
  const video = asset.mediaInfo?.videoStream ?? null;
  if (video === null) {
    return null;
  }
  const size = displaySize(video);
  const fps = framesPerSecond(video);
  return (
    <PropertyGroup title={t('editor.media.info.video')}>
      <PropertyRow label={t('editor.media.info.codec')}>
        <Value>{codecLabel(video.codec, video.profile)}</Value>
      </PropertyRow>
      <PropertyRow label={t('editor.media.info.resolution')}>
        <Value>
          {video.rotation === 0
            ? t('editor.media.info.resolutionValue', size)
            : t('editor.media.info.rotated', { ...size, degrees: video.rotation })}
        </Value>
      </PropertyRow>
      {asset.kind === 'video' && (
        <PropertyRow label={t('editor.media.info.frameRate')}>
          <Value>
            {fps === null
              ? t('editor.media.info.variableFrameRate')
              : t('editor.media.info.frameRateValue', { fps })}
          </Value>
        </PropertyRow>
      )}
      {video.bitDepth !== null && (
        <PropertyRow label={t('editor.media.info.depth')}>
          <Value>
            {t('editor.media.info.depthValue', {
              bits: video.bitDepth,
              chroma: video.chromaSubsampling ?? '',
            })}
          </Value>
        </PropertyRow>
      )}
      <PropertyRow label={t('editor.media.info.decodability')}>
        <Value>{decodability(t, asset.decodability.video)}</Value>
      </PropertyRow>
    </PropertyGroup>
  );
}
