import type { Animatable, Flicks, ItemEntity, MediaClip, TrackKind } from '@se/protocol';
import { cn } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import type { ItemSilence } from '../services/audio-edits';
import type { ItemBox } from '../services/item-geometry';
import type { ClipEdit } from '../services/preview';
import {
  MIN_VOLUME_BAND_HEIGHT,
  formatVolume,
  volumeBand,
  volumeLinePoints,
  volumeToFraction,
} from '../services/volume-line';

/** Props of {@link ClipOverlay}. */
export interface ClipOverlayProps {
  readonly item: ItemEntity;
  readonly clip: MediaClip;
  readonly trackKind: TrackKind;
  readonly box: ItemBox;
  readonly height: number;
  readonly pixelsPerSecond: number;
  /** Fade handles and the volume curve can be dragged (the track is not locked). */
  readonly editable: boolean;
  /** Levels a gesture is dragging, drawn instead of the document's. */
  readonly edit: ClipEdit | undefined;
  /** Silences to hatch (audio tracks, when shown). */
  readonly silences: readonly ItemSilence[];
}

/** The `points` attribute of an SVG polygon or polyline. */
function svgPoints(...points: (readonly [number, number])[]): string {
  return points.map(([x, y]) => [x, y].join(',')).join(' ');
}

/** Narrower clips show no fade handles. */
const MIN_HANDLE_WIDTH = 24;

/**
 * What a media clip shows over its pictures or waveform: its fades (shaded ramps with a handle at
 * each top corner), on audio tracks its volume line with keyframes (`data-volume-line`,
 * `data-volume-keyframe`) and the silences of its sound. The lanes handle the pointer.
 */
export function ClipOverlay(props: ClipOverlayProps) {
  const { item, clip, box, height, pixelsPerSecond, edit } = props;
  const x = (time: number) => (time / FLICKS_PER_SECOND) * pixelsPerSecond - box.offsetPixels;
  const duration = item.timelineRange.duration;
  const fadeIn = edit?.fadeIn ?? clip.fadeIn;
  const fadeOut = edit?.fadeOut ?? clip.fadeOut;
  const band = volumeBand(height);
  const showVolume = props.trackKind === 'audio' && band.height >= MIN_VOLUME_BAND_HEIGHT;
  const handles = props.editable && box.width >= MIN_HANDLE_WIDTH;
  const bottom = height - 2;
  return (
    <svg
      aria-hidden
      className="pointer-events-none absolute inset-0 z-10 h-full w-full overflow-hidden"
    >
      {props.silences.map((silence) => (
        <rect
          key={silence.start}
          data-silence
          x={x(silence.start)}
          y={band.top}
          width={Math.max(1, x(silence.end) - x(silence.start))}
          height={band.height}
          className="fill-text/15 stroke-text/30"
          strokeDasharray="2 3"
        />
      ))}
      {fadeIn > 0 && (
        <polygon
          data-fade="in"
          points={svgPoints([x(0), 1], [x(fadeIn), 1], [x(0), bottom])}
          className="fill-bg/55 stroke-text/70"
        />
      )}
      {fadeOut > 0 && (
        <polygon
          data-fade="out"
          points={svgPoints([x(duration), 1], [x(duration - fadeOut), 1], [x(duration), bottom])}
          className="fill-bg/55 stroke-text/70"
        />
      )}
      {showVolume && (
        <VolumeLine
          volume={edit?.volumeDb ?? clip.volumeDb}
          duration={duration}
          x={x}
          band={band}
          editable={props.editable}
        />
      )}
      {handles && box.startVisible && <FadeHandle edge="in" x={x(fadeIn)} />}
      {handles && box.endVisible && <FadeHandle edge="out" x={x(duration - fadeOut)} />}
    </svg>
  );
}

/** The square at a top corner of a clip that sizes its fade. */
function FadeHandle({ edge, x }: { readonly edge: 'in' | 'out'; readonly x: number }) {
  const size = 8;
  return (
    <rect
      data-fade-handle={edge}
      x={edge === 'in' ? x : x - size}
      y={1}
      width={size}
      height={size}
      rx={1}
      className="pointer-events-auto cursor-ew-resize fill-text stroke-bg"
    />
  );
}

interface VolumeLineProps {
  readonly volume: Animatable<number>;
  readonly duration: Flicks;
  readonly x: (time: number) => number;
  readonly band: { readonly top: number; readonly height: number };
  readonly editable: boolean;
}

/** The clip's volume as a line over its waveform, keyframes as dots. */
function VolumeLine({ volume, duration, x, band, editable }: VolumeLineProps) {
  const { t } = useTranslation();
  const y = (db: number) => band.top + (1 - volumeToFraction(db)) * band.height;
  const points = svgPoints(
    ...volumeLinePoints(volume, duration).map((point) => [x(point.time), y(point.db)] as const),
  );
  const label =
    volume.kind === 'static'
      ? t('editor.timeline.volume.static', { volume: formatVolume(volume.value) })
      : t('editor.timeline.volume.keyframed');
  return (
    <g data-volume>
      <polyline points={points} className="fill-none stroke-warning" strokeWidth={1.5} />
      {editable && (
        <polyline
          data-volume-line
          points={points}
          className="pointer-events-auto cursor-ns-resize fill-none stroke-transparent"
          strokeWidth={8}
        >
          <title>{label}</title>
        </polyline>
      )}
      {volume.kind === 'keyframed' &&
        volume.keyframes.map((keyframe, index) => (
          <circle
            key={keyframe.time}
            data-volume-keyframe={index}
            cx={x(keyframe.time)}
            cy={y(keyframe.value)}
            r={editable ? 4 : 3}
            className={cn('fill-warning stroke-bg', editable && 'pointer-events-auto cursor-move')}
          >
            <title>{formatVolume(keyframe.value)}</title>
          </circle>
        ))}
    </g>
  );
}
