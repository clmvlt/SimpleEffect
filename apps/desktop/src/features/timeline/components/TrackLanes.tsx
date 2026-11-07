import type { ItemId } from '@se/protocol';
import { useRef, type RefObject } from 'react';

import { useElementWidth } from '../hooks/use-element-width';
import { useLaneSelection } from '../hooks/use-lane-selection';
import type { TimelineModel } from '../hooks/use-timeline-model';
import type { TimeSpan } from '../services/in-out';
import { OVERSCAN_PIXELS, spanBox } from '../services/item-geometry';
import { marqueeBox } from '../services/marquee';
import { allRows, SECTION_DIVIDER_HEIGHT, type TrackLayout } from '../services/track-layout';
import { visibleRange, type Viewport } from '../services/viewport';
import { visibleItems } from '../services/visible-items';
import { setLanesWidth } from '../store/timeline-view-store';
import { PlayheadLine } from './PlayheadLine';
import { TrackLane } from './TrackLane';

/** Props of {@link TrackLanes}. */
export interface TrackLanesProps {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
  readonly selected: ReadonlySet<ItemId>;
  /** The in/out span to shade, if any. */
  readonly span: TimeSpan | null;
  /** Id of the lanes element (the minimap scrolls it). */
  readonly id: string;
}

/**
 * The lanes of every track (only items in view are rendered), the in/out shade, the playhead line
 * and the selection rectangle. Its measured width is the viewport width.
 */
export function TrackLanes({ model, layout, viewport, selected, span, id }: TrackLanesProps) {
  const element: RefObject<HTMLDivElement | null> = useRef<HTMLDivElement>(null);
  useElementWidth(element, setLanesWidth);
  const { rect, onItemPointerDown, laneHandlers } = useLaneSelection(model, layout, viewport);
  const range = visibleRange(viewport, OVERSCAN_PIXELS);
  const box = rect === null ? null : marqueeBox(rect);
  return (
    <div
      ref={element}
      id={id}
      data-timeline-lanes
      className="relative min-w-0 flex-1 overflow-hidden"
      {...laneHandlers}
    >
      {allRows(layout).map((row) => (
        <TrackLane
          key={row.track.id}
          row={row}
          items={visibleItems(model.itemsByTrack.get(row.track.id) ?? [], range.start, range.end)}
          viewport={viewport}
          assets={model.assets}
          templates={model.templates}
          selected={selected}
          onItemPointerDown={onItemPointerDown}
        />
      ))}
      <div
        aria-hidden
        className="absolute inset-x-0 border-y border-border bg-panel-raised"
        style={{ top: layout.dividerTop, height: SECTION_DIVIDER_HEIGHT }}
      />
      {span !== null && (
        <div
          aria-hidden
          data-in-out-shade
          className="pointer-events-none absolute inset-y-0 z-10 border-x border-accent/60 bg-accent-muted/50"
          style={spanBox(span, viewport)}
        />
      )}
      <PlayheadLine sequenceId={model.sequence.id} viewport={viewport} />
      {box !== null && (
        <div
          aria-hidden
          data-marquee
          className="pointer-events-none absolute z-30 rounded-xs border border-accent bg-accent-muted/30"
          style={box}
        />
      )}
    </div>
  );
}
