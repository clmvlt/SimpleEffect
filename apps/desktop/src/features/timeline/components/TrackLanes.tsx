import type { ItemId } from '@se/protocol';
import { ContextMenu, ContextMenuTrigger } from '@se/ui';
import { useRef, type RefObject } from 'react';

import type { GestureContext } from '../hooks/gestures/gesture-context';
import type { EditActions } from '../hooks/use-edit-actions';
import type { TransitionActions } from '../hooks/use-transition-actions';
import { useElementWidth } from '../hooks/use-element-width';
import { useLaneMenu } from '../hooks/use-lane-menu';
import { useLanePointer } from '../hooks/use-lane-pointer';
import { useMediaDrop } from '../hooks/use-media-drop';
import type { TimeSpan } from '../services/in-out';
import { OVERSCAN_PIXELS, spanBox } from '../services/item-geometry';
import { marqueeBox } from '../services/marquee';
import { allRows, SECTION_DIVIDER_HEIGHT } from '../services/track-layout';
import { visibleRange } from '../services/viewport';
import { visibleItems } from '../services/visible-items';
import { clipboardFor } from '../store/clipboard-store';
import { useGesturePreview } from '../store/gesture-store';
import { useEditPreferences } from '../store/edit-preferences-store';
import { setLanesWidth } from '../store/timeline-view-store';
import { AiScopeOverlays } from './AiScopeOverlays';
import { CutPreviewOverlay } from './CutPreviewOverlay';
import { GapHighlight } from './GapHighlight';
import { GapHover, useGapHover } from './GapHover';
import { GestureOverlay } from './GestureOverlay';
import { LaneMenu } from './LaneMenu';
import { PlayheadLine } from './PlayheadLine';
import { TrackLane } from './TrackLane';
import { TransitionMarks } from './TransitionMarks';

/** Props of {@link TrackLanes}. */
export interface TrackLanesProps {
  /** The model, layout and viewport (and what gestures read), always the latest. */
  readonly context: () => GestureContext;
  readonly actions: EditActions;
  readonly transitionActions: TransitionActions;
  readonly selected: ReadonlySet<ItemId>;
  /** The in/out span to shade, if any. */
  readonly span: TimeSpan | null;
  /** Id of the lanes element (the minimap scrolls it). */
  readonly id: string;
}

const NONE: ReadonlySet<ItemId> = new Set();

/**
 * The lanes of every track (only items in view are rendered), the in/out shade, the hovered and
 * selected gaps, the transitions and cut handles, the scopes of active AI tasks, the playhead
 * line, the selection rectangle and what a gesture previews. They take the pointer (selection, drags, trims, gap handle, fades,
 * volume curves, transitions), media dropped from the bin and right clicks.
 * Their measured width is the viewport width.
 */
export function TrackLanes({
  context,
  actions,
  transitionActions,
  selected,
  span,
  id,
}: TrackLanesProps) {
  const element: RefObject<HTMLDivElement | null> = useRef<HTMLDivElement>(null);
  useElementWidth(element, setLanesWidth);
  const { model, layout, viewport, commands } = context();
  const { rect, laneHandlers } = useLanePointer(context);
  const gapHover = useGapHover(context);
  const drop = useMediaDrop(context);
  const menu = useLaneMenu(context);
  const preview = useGesturePreview();
  const { showSilences } = useEditPreferences();
  const hidden = preview === null ? NONE : new Set(preview.placements.keys());
  const range = visibleRange(viewport, OVERSCAN_PIXELS);
  const box = rect === null ? null : marqueeBox(rect);
  return (
    <ContextMenu>
      <ContextMenuTrigger asChild>
        <div
          ref={element}
          id={id}
          data-timeline-lanes
          className="relative min-w-0 flex-1 overflow-hidden"
          onContextMenu={menu.onContextMenu}
          {...laneHandlers}
          onPointerMove={(event) => {
            laneHandlers.onPointerMove(event);
            gapHover.onPointerMove(event);
          }}
          onPointerLeave={gapHover.onPointerLeave}
          {...drop}
        >
          {allRows(layout).map((row) => (
            <TrackLane
              key={row.track.id}
              row={row}
              items={visibleItems(
                model.itemsByTrack.get(row.track.id) ?? [],
                range.start,
                range.end,
              )}
              viewport={viewport}
              assets={model.assets}
              templates={model.templates}
              selected={selected}
              hidden={hidden}
              edits={preview?.clipEdits}
              showSilences={showSilences}
              keyframes={{ commands, frameRate: model.sequence.format.frameRate }}
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
          <GapHover
            gap={preview === null ? gapHover.hovered : null}
            layout={layout}
            viewport={viewport}
          />
          <GapHighlight model={model} layout={layout} viewport={viewport} />
          <TransitionMarks model={model} layout={layout} viewport={viewport} />
          <AiScopeOverlays sequenceId={model.sequence.id} layout={layout} viewport={viewport} />
          <CutPreviewOverlay sequenceId={model.sequence.id} viewport={viewport} />
          <GestureOverlay model={model} layout={layout} viewport={viewport} />
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
      </ContextMenuTrigger>
      <LaneMenu
        target={menu.target}
        actions={actions}
        transitionActions={transitionActions}
        canPaste={clipboardFor(model.projectId) !== null}
      />
    </ContextMenu>
  );
}
