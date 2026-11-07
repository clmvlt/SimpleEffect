import type { Flicks, MarkerEntity, MarkerPatch } from '@se/protocol';
import { useTranslation } from 'react-i18next';

import { useRulerPointer, type RulerPointerTargets } from '../hooks/use-ruler-pointer';
import type { TimeSpan } from '../services/in-out';
import { spanBox } from '../services/item-geometry';
import { timeToX } from '../services/viewport';
import { MarkerFlag } from './MarkerFlag';
import { PlayheadLine } from './PlayheadLine';
import { RulerCanvas } from './RulerCanvas';

/** Height of the ruler in pixels. */
export const RULER_HEIGHT = 32;

/** Props of {@link TimeRuler}. */
export interface TimeRulerProps extends RulerPointerTargets {
  readonly sequenceId: string;
  readonly markers: readonly MarkerEntity[];
  readonly span: TimeSpan | null;
  readonly onMarkerChange: (marker: MarkerEntity, patch: MarkerPatch) => void;
  readonly onMarkerRemove: (marker: MarkerEntity) => void;
  readonly onMarkerGoTo: (time: Flicks) => void;
}

/**
 * The timecode ruler: graduations adapted to the zoom, the in/out span, markers and the playhead
 * handle. Press or drag to move the playhead; Maj+drag to mark in and out.
 */
export function TimeRuler({
  sequenceId,
  markers,
  span,
  onMarkerChange,
  onMarkerRemove,
  onMarkerGoTo,
  ...targets
}: TimeRulerProps) {
  const { t } = useTranslation();
  const { viewport, frameRate } = targets;
  const handlers = useRulerPointer(targets);
  const spanRect = span === null ? null : spanBox(span, viewport);
  return (
    <div
      role="group"
      aria-label={t('editor.timeline.ruler')}
      data-timeline-ruler
      className="relative min-w-0 flex-1 cursor-text overflow-hidden border-b border-border bg-panel"
      style={{ height: RULER_HEIGHT }}
      {...handlers}
    >
      <RulerCanvas viewport={viewport} frameRate={frameRate} height={RULER_HEIGHT} />
      {spanRect !== null && (
        <div
          aria-hidden
          data-in-out-span
          className="pointer-events-none absolute bottom-0 h-2.5 rounded-t-xs border-x-2 border-accent bg-accent-muted"
          style={spanRect}
        />
      )}
      {markers.map((marker) => (
        <MarkerFlag
          key={marker.id}
          marker={marker}
          x={timeToX(marker.time, viewport)}
          onGoTo={() => {
            onMarkerGoTo(marker.time);
          }}
          onChange={(patch) => {
            onMarkerChange(marker, patch);
          }}
          onRemove={() => {
            onMarkerRemove(marker);
          }}
        />
      ))}
      <PlayheadLine sequenceId={sequenceId} viewport={viewport}>
        <div className="absolute bottom-0 left-1/2 h-2.5 w-2.5 -translate-x-1/2 rounded-b-xs bg-accent" />
      </PlayheadLine>
    </div>
  );
}
