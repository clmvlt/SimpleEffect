import type { Flicks, TrackEntity, TransitionId } from '@se/protocol';
import { cn } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { formatDuration } from '../../../shared/time';
import type { TimelineModel } from '../hooks/use-timeline-model';
import { openCuts, transitionMarks } from '../services/audio-edits';
import { rangeBox, OVERSCAN_PIXELS } from '../services/item-geometry';
import type { TransitionDraft } from '../services/preview';
import { allRows, type TrackLayout, type TrackRow } from '../services/track-layout';
import { timeToX, visibleRange, type Viewport } from '../services/viewport';
import { useGesturePreview } from '../store/gesture-store';
import { useSelectedTransition } from '../store/selection-store';

/** Props of {@link TransitionMarks}. */
export interface TransitionMarksProps {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
}

/** Height of the band of a row where transitions are drawn (its lower half, at most 22 px). */
function markBand(row: TrackRow): { readonly top: number; readonly height: number } {
  const height = Math.min(22, Math.max(10, Math.round(row.height / 2) - 2));
  return { top: row.top + row.height - height - 2, height };
}

/**
 * The transitions of every track in view, drawn over their cut (`data-transition-id`, edges
 * `data-transition-edge` resize them, a click selects), the transition a gesture is drawing, and
 * a handle on each cut without transition (`data-cut-left`: drag to size a cross-fade, click for
 * one of the default length).
 */
export function TransitionMarks({ model, layout, viewport }: TransitionMarksProps) {
  const draft = useGesturePreview()?.transition;
  const selected = useSelectedTransition();
  const range = visibleRange(viewport, OVERSCAN_PIXELS);
  return (
    <>
      {allRows(layout).map((row) => {
        const items = model.itemsByTrack.get(row.track.id) ?? [];
        const band = markBand(row);
        return (
          <div key={row.track.id} className="contents">
            {transitionMarks(row.track, items)
              .filter((mark) => mark.span.end >= range.start && mark.span.start <= range.end)
              .map(({ transition, span }) => {
                const shown = draft?.transitionId === transition.id ? draft : span;
                return (
                  <TransitionBox
                    key={transition.id}
                    track={row.track}
                    transitionId={transition.id}
                    kind={transition.kind}
                    start={shown.start}
                    end={shown.end}
                    band={band}
                    viewport={viewport}
                    selected={selected === transition.id}
                  />
                );
              })}
            {!row.track.locked &&
              openCuts(row.track, items)
                .filter((cut) => cut.time >= range.start && cut.time <= range.end)
                .map((cut) => (
                  <CutHandle
                    key={cut.left.id}
                    track={row.track}
                    leftItem={cut.left.id}
                    x={timeToX(cut.time, viewport)}
                    bottom={row.top + row.height}
                  />
                ))}
          </div>
        );
      })}
      {draft !== undefined && draft.transitionId === null && (
        <DraftBox draft={draft} layout={layout} viewport={viewport} />
      )}
    </>
  );
}

interface TransitionBoxProps {
  readonly track: TrackEntity;
  readonly transitionId: TransitionId;
  readonly kind: 'crossDissolve' | 'audioCrossfade';
  readonly start: Flicks;
  readonly end: Flicks;
  readonly band: { readonly top: number; readonly height: number };
  readonly viewport: Viewport;
  readonly selected: boolean;
}

function TransitionBox(props: TransitionBoxProps) {
  const { t } = useTranslation();
  const box = rangeBox(props.start, props.end, props.viewport);
  const duration = formatDuration((props.end - props.start) as Flicks);
  const name = t(`editor.timeline.transition.kinds.${props.kind}`);
  return (
    <div
      role="button"
      aria-pressed={props.selected}
      aria-label={t('editor.timeline.transition.label', { name, duration })}
      title={t('editor.timeline.transition.label', { name, duration })}
      data-transition-id={props.transitionId}
      data-track-id={props.track.id}
      className={cn(
        'absolute z-20 flex items-center justify-center overflow-hidden rounded-xs border bg-panel-raised/90 text-2xs text-text-muted',
        props.selected ? 'border-accent ring-1 ring-accent' : 'border-border-strong',
      )}
      style={{ left: box.left, width: box.width, top: props.band.top, height: props.band.height }}
    >
      <svg aria-hidden className="h-full w-full" preserveAspectRatio="none" viewBox="0 0 10 10">
        <line x1={0} y1={10} x2={10} y2={0} className="stroke-text-subtle" strokeWidth={0.6} />
        <line x1={0} y1={0} x2={10} y2={10} className="stroke-text-subtle" strokeWidth={0.6} />
      </svg>
      {box.startVisible && <TransitionEdge edge="start" />}
      {box.endVisible && <TransitionEdge edge="end" />}
    </div>
  );
}

function TransitionEdge({ edge }: { readonly edge: 'start' | 'end' }) {
  return (
    <div
      aria-hidden
      data-transition-edge={edge}
      className={cn(
        'absolute inset-y-0 w-1.5 cursor-ew-resize hover:bg-accent/60',
        edge === 'start' ? 'left-0' : 'right-0',
      )}
    />
  );
}

interface CutHandleProps {
  readonly track: TrackEntity;
  readonly leftItem: string;
  readonly x: number;
  readonly bottom: number;
}

/** The small diamond at the foot of a cut. */
function CutHandle({ track, leftItem, x, bottom }: CutHandleProps) {
  const { t } = useTranslation();
  const size = 10;
  return (
    <div
      role="button"
      aria-label={t('editor.timeline.transition.cutHandle')}
      title={t('editor.timeline.transition.cutHandle')}
      data-cut-left={leftItem}
      data-track-id={track.id}
      className="absolute z-20 rotate-45 cursor-ew-resize rounded-xs border border-bg bg-text-subtle opacity-60 hover:bg-accent hover:opacity-100"
      style={{ left: x - size / 2, top: bottom - size - 3, width: size, height: size }}
    />
  );
}

/** A new cross-fade being dragged from a cut. */
function DraftBox(props: {
  readonly draft: TransitionDraft;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
}) {
  const row = allRows(props.layout).find((candidate) => candidate.track.id === props.draft.trackId);
  if (row === undefined) return null;
  const band = markBand(row);
  const box = rangeBox(props.draft.start, props.draft.end, props.viewport);
  return (
    <div
      aria-hidden
      data-transition-draft
      className="pointer-events-none absolute z-20 rounded-xs border border-dashed border-accent bg-accent-muted"
      style={{ left: box.left, width: box.width, top: band.top, height: band.height }}
    />
  );
}
