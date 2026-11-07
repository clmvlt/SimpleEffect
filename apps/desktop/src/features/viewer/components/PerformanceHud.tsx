import type { SequenceEntity } from '@se/protocol';
import { useTranslation } from 'react-i18next';

import { byteSize } from '../../../shared/text';
import { usePerformanceStats } from '../hooks/use-performance-stats';
import type { PlaybackSession } from '../services/playback-session';
import type { ViewerEngineController } from '../services/viewer-engine-controller';

/** Props of {@link PerformanceHud}. */
export interface PerformanceHudProps {
  readonly engine: ViewerEngineController;
  readonly session: PlaybackSession;
  readonly sequence: SequenceEntity;
}

/**
 * The performance HUD over the picture (toggled from the viewer toolbar): real frames per second
 * against the sequence rate, frames dropped, decoding work, picture and sound caches.
 */
export function PerformanceHud({ engine, session, sequence }: PerformanceHudProps) {
  const { t, i18n } = useTranslation();
  const { picture, audio } = usePerformanceStats(engine, session);
  const { width, height, frameRate } = sequence.format;
  const number = new Intl.NumberFormat(i18n.language, { maximumFractionDigits: 3 });
  const fps = number.format(frameRate.numerator / frameRate.denominator);
  return (
    <div
      role="group"
      aria-label={t('editor.viewer.hud.label')}
      data-performance-hud=""
      className="pointer-events-none absolute top-2 left-2 flex flex-col gap-0.5 rounded-sm bg-overlay px-2 py-1 timecode text-2xs text-text-muted"
    >
      <span>{t('editor.viewer.format', { width, height, fps })}</span>
      {picture !== null && (
        <>
          <span className="text-text">
            {t('editor.viewer.hud.fps', { fps: number.format(picture.framesPerSecond) })}
          </span>
          <span>{t('editor.viewer.hud.dropped', { count: picture.droppedFrames })}</span>
          <span>
            {t('editor.viewer.hud.decoding', {
              readers: picture.readers,
              queued: picture.queuedDecodes,
            })}
          </span>
          <span>
            {t('editor.viewer.hud.cache', {
              size: byteSize(picture.cachedBytes, i18n.language),
              count: picture.cachedPictures,
            })}
          </span>
          <span>
            {t('editor.viewer.hud.prefetch', { seconds: number.format(picture.prefetchSeconds) })}
          </span>
        </>
      )}
      {audio !== null && (
        <span>
          {t('editor.viewer.hud.audio', {
            size: byteSize(audio.bytes, i18n.language),
            voices: audio.voices,
          })}
        </span>
      )}
    </div>
  );
}
