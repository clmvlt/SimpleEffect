import { useTranslation } from 'react-i18next';

import { formatDuration } from '../../../shared/time';
import type { TimelineModel } from '../hooks/use-timeline-model';
import { gapDuration, isGapOf } from '../services/gaps';
import { rangeBox } from '../services/item-geometry';
import { allRows, type TrackLayout } from '../services/track-layout';
import type { Viewport } from '../services/viewport';
import { useGesturePreview } from '../store/gesture-store';
import { useSelectedGap } from '../store/selection-store';

/** Props of {@link GapHighlight}. */
export interface GapHighlightProps {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
}

/**
 * The selected gap: shaded on its track with its length, and a handle on its end that lengthens
 * or shortens it (what follows moves). `Suppr` closes it.
 */
export function GapHighlight({ model, layout, viewport }: GapHighlightProps) {
  const { t } = useTranslation();
  const selected = useSelectedGap();
  const preview = useGesturePreview();
  if (selected === null) return null;
  const row = allRows(layout).find((candidate) => candidate.track.id === selected.trackId);
  if (row === undefined || !isGapOf(model.itemsByTrack.get(selected.trackId) ?? [], selected)) {
    return null;
  }
  const gap = preview?.gap ?? selected;
  const box = rangeBox(gap.start, gap.end, viewport);
  const duration = formatDuration(gapDuration(gap));
  return (
    <div
      data-gap
      role="status"
      aria-label={t('editor.timeline.gap.label', { duration })}
      className="pointer-events-none absolute z-10 rounded-xs border border-dashed border-accent bg-accent-muted"
      style={{ left: box.left, width: box.width, top: row.top + 2, height: row.height - 4 }}
    >
      <span className="absolute inset-x-1 top-0.5 truncate text-center text-2xs text-text timecode">
        {duration}
      </span>
      {box.endVisible && (
        <div
          aria-hidden
          data-gap-handle
          className="pointer-events-auto absolute inset-y-0 -right-1 w-2 cursor-ew-resize rounded-xs bg-accent"
          title={t('editor.timeline.gap.handle')}
        />
      )}
    </div>
  );
}
