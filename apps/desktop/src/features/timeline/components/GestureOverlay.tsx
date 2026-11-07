import type { Flicks } from '@se/protocol';
import { ClipShell, cn } from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { TimelineModel } from '../hooks/use-timeline-model';
import { itemDisplay, type ItemTone } from '../services/item-display';
import { rangeBox } from '../services/item-geometry';
import type { PreviewGhost, PreviewPlacement } from '../services/preview';
import { DEFAULT_TRACK_HEIGHT } from '../services/track-height';
import { allRows, type TrackLayout, type TrackRow } from '../services/track-layout';
import { timeToX, visibleRange, type Viewport } from '../services/viewport';
import { useGesturePreview } from '../store/gesture-store';

/** Props of {@link GestureOverlay}. */
export interface GestureOverlayProps {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
}

/** Vertical position of a row, or of the drop zone below the last track. */
type RowBox = Pick<TrackRow, 'top' | 'height'> & { readonly kind: 'video' | 'audio' };

/** One clip drawn by a gesture: an item where it would go, or a ghost of a new one. */
function PreviewClip({
  row,
  start,
  duration,
  tone,
  label,
  viewport,
  ghost,
}: {
  readonly row: RowBox;
  readonly start: Flicks;
  readonly duration: Flicks;
  readonly tone: ItemTone;
  readonly label: string;
  readonly viewport: Viewport;
  readonly ghost: boolean;
}) {
  const box = rangeBox(start, (start + duration) as Flicks, viewport);
  return (
    <div
      data-preview-clip={ghost ? 'ghost' : 'placement'}
      className={cn('absolute py-px', ghost && 'opacity-70')}
      style={{ left: box.left, width: box.width, top: row.top, height: row.height }}
    >
      <ClipShell tone={tone} label={label} selected />
    </div>
  );
}

/**
 * What the gesture in progress shows over the lanes (ephemeral, never sent to the core): items
 * where they would go, ghosts of new items, and the line of the point an edge snapped to.
 */
export function GestureOverlay({ model, layout, viewport }: GestureOverlayProps) {
  const { t } = useTranslation();
  const preview = useGesturePreview();
  if (preview === null) return null;
  const rows = new Map(allRows(layout).map((row) => [row.track.id, row]));
  const newAudioRow: RowBox = { top: layout.height, height: DEFAULT_TRACK_HEIGHT, kind: 'audio' };
  const range = visibleRange(viewport, 200);
  const inView = (start: Flicks, duration: Flicks) =>
    start <= range.end && start + duration >= range.start;
  const labelOf = (label: string | null) => label ?? t('editor.timeline.items.media');

  const placement = ([itemId, place]: readonly [string, PreviewPlacement]) => {
    const item = model.itemsById[itemId];
    const row = rows.get(place.trackId);
    if (item === undefined || row === undefined || !inView(place.start, place.duration)) return [];
    const display = itemDisplay(item, row.track.kind, model.assets, model.templates);
    return [{ key: itemId, row: { ...row, kind: row.track.kind }, place, display }];
  };
  const ghost = (entry: PreviewGhost) => {
    const row = entry.row.kind === 'newAudio' ? newAudioRow : rows.get(entry.row.trackId);
    if (row === undefined || !inView(entry.start, entry.duration)) return [];
    const box: RowBox = 'track' in row ? { ...row, kind: row.track.kind } : row;
    if (entry.source.kind === 'media') {
      return [{ key: entry.key, row: box, tone: entry.source.tone, label: entry.source.label }];
    }
    const item = model.itemsById[entry.source.itemId];
    if (item === undefined) return [];
    const display = itemDisplay(item, box.kind, model.assets, model.templates);
    return [{ key: entry.key, row: box, tone: display.tone, label: labelOf(display.label) }];
  };

  return (
    <div aria-hidden className="pointer-events-none absolute inset-0 z-20">
      {[...preview.placements].flatMap(placement).map(({ key, row, place, display }) => (
        <PreviewClip
          key={key}
          row={row}
          start={place.start}
          duration={place.duration}
          tone={display.tone}
          label={labelOf(display.label)}
          viewport={viewport}
          ghost={false}
        />
      ))}
      {preview.ghosts.flatMap((entry) =>
        ghost(entry).map(({ key, row, tone, label }) => (
          <PreviewClip
            key={key}
            row={row}
            start={entry.start}
            duration={entry.duration}
            tone={tone}
            label={label}
            viewport={viewport}
            ghost
          />
        )),
      )}
      {preview.snapTime !== null && (
        <div
          data-snap-line
          className="absolute inset-y-0 w-px bg-accent"
          style={{ left: timeToX(preview.snapTime, viewport) }}
        />
      )}
    </div>
  );
}
