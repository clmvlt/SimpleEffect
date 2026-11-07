import type { PointerEvent } from 'react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import type { GestureContext } from '../hooks/gestures/gesture-context';
import { gapAt, gapDuration, type Gap } from '../services/gaps';
import { rangeBox } from '../services/item-geometry';
import { allRows, rowAt, type TrackLayout } from '../services/track-layout';
import { xToTime, type Viewport } from '../services/viewport';

/** A duration in seconds with two decimals (`1,25 s`), precise enough to adjust a gap. */
export function formatSeconds(flicks: number): string {
  return `${(flicks / FLICKS_PER_SECOND).toFixed(2).replace('.', ',')} s`;
}

function sameGap(left: Gap | null, right: Gap | null): boolean {
  return (
    left === right ||
    (left?.trackId === right?.trackId && left?.start === right?.start && left?.end === right?.end)
  );
}

/**
 * The gap under the pointer while nothing is pressed (`onPointerMove`/`onPointerLeave` go on the
 * lanes): the lanes outline it with its duration, so blanks left by a cut are easy to see.
 */
export function useGapHover(context: () => GestureContext) {
  const [hovered, setHovered] = useState<Gap | null>(null);
  const update = (next: Gap | null) => {
    setHovered((current) => (sameGap(current, next) ? current : next));
  };
  const onPointerMove = (event: PointerEvent<HTMLElement>) => {
    const onItem = event.target instanceof Element && event.target.closest('[data-item-id]');
    if (event.buttons !== 0 || onItem) {
      update(null);
      return;
    }
    const { model, layout, viewport } = context();
    const bounds = event.currentTarget.getBoundingClientRect();
    const row = rowAt(layout, event.clientY - bounds.top);
    if (row === undefined) {
      update(null);
      return;
    }
    const time = xToTime(event.clientX - bounds.left, viewport);
    update(gapAt(model.itemsByTrack.get(row.track.id) ?? [], row.track.id, time));
  };
  return {
    hovered,
    onPointerMove,
    onPointerLeave: () => {
      update(null);
    },
  };
}

/** Props of {@link GapHover}. */
export interface GapHoverProps {
  readonly gap: Gap | null;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
}

/** The outline of the hovered gap with its duration (a click selects it, `Suppr` closes it). */
export function GapHover({ gap, layout, viewport }: GapHoverProps) {
  const { t } = useTranslation();
  if (gap === null) return null;
  const row = allRows(layout).find((candidate) => candidate.track.id === gap.trackId);
  if (row === undefined) return null;
  const box = rangeBox(gap.start, gap.end, viewport);
  const duration = formatSeconds(gapDuration(gap));
  return (
    <div
      data-gap-hover
      role="status"
      aria-label={t('editor.timeline.gap.hover', { duration })}
      className="pointer-events-none absolute z-10 rounded-xs border border-dashed border-border-strong bg-hover"
      style={{ left: box.left, width: box.width, top: row.top + 2, height: row.height - 4 }}
    >
      <span className="absolute inset-x-1 top-1/2 -translate-y-1/2 truncate text-center text-2xs text-text-muted timecode">
        {duration}
      </span>
    </div>
  );
}
