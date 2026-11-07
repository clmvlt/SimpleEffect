import { useTrackDrags } from '../hooks/use-track-drags';
import type { TimelineCommands } from '../hooks/use-timeline-commands';
import type { TimelineModel } from '../hooks/use-timeline-model';
import {
  allRows,
  insertionIndex,
  SECTION_DIVIDER_HEIGHT,
  type TrackLayout,
} from '../services/track-layout';
import { TrackHeader } from './TrackHeader';

/** Width of the track header column in pixels. */
export const TRACK_HEADER_WIDTH = 208;

/** Props of {@link TrackHeaderList}. */
export interface TrackHeaderListProps {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly commands: TimelineCommands;
}

/** The column of track headers, rows aligned with the lanes, with the reorder drop line. */
export function TrackHeaderList({ model, layout, commands }: TrackHeaderListProps) {
  const { grip, resizeHandle, draggedTrackId, dropLine } = useTrackDrags(model, layout, commands);
  const { sequence } = model;
  return (
    <div
      data-track-headers
      className="relative shrink-0 border-r border-border bg-panel"
      style={{ width: TRACK_HEADER_WIDTH }}
    >
      {allRows(layout).map((row) => (
        <TrackHeader
          key={row.track.id}
          row={row}
          dragging={draggedTrackId === row.track.id}
          gripHandlers={grip(row.track)}
          resizeHandlers={resizeHandle(row.track, row.height)}
          onChange={(patch) => {
            void commands.updateTrack(row.track.id, patch);
          }}
          onAdd={(placement) => {
            const index = insertionIndex(sequence.trackIds, row.track, placement);
            void commands.addTrack(sequence.id, row.track.kind, index);
          }}
          onRemove={() => {
            void commands.removeTrack(row.track.id);
          }}
        />
      ))}
      <div
        aria-hidden
        className="absolute inset-x-0 border-y border-border bg-panel-raised"
        style={{ top: layout.dividerTop, height: SECTION_DIVIDER_HEIGHT }}
      />
      {dropLine !== null && (
        <div
          aria-hidden
          data-drop-line
          className="pointer-events-none absolute inset-x-0 z-10 h-0.5 bg-accent"
          style={{ top: dropLine - 1 }}
        />
      )}
    </div>
  );
}
