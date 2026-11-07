import type { Flicks } from '@se/protocol';
import { useId, useRef } from 'react';
import { useStore } from 'zustand';

import { useInOutPoints } from '../hooks/use-in-out';
import type { InOutControls } from '../hooks/use-in-out';
import type { PlayheadControls } from '../hooks/use-playhead-controls';
import type { TimelineCommands } from '../hooks/use-timeline-commands';
import type { TimelineModel } from '../hooks/use-timeline-model';
import type { TimelineViewportApi } from '../hooks/use-timeline-viewport';
import { useTimelineWheel } from '../hooks/use-timeline-wheel';
import { inOutSpan } from '../services/in-out';
import { layoutTracks } from '../services/track-layout';
import { DEFAULT_TRACK_HEIGHT } from '../services/track-height';
import { useSelectedItemIds } from '../store/selection-store';
import { timelineViewStore } from '../store/timeline-view-store';
import { Minimap } from './Minimap';
import { TimeRuler } from './TimeRuler';
import { TRACK_HEADER_WIDTH, TrackHeaderList } from './TrackHeaderList';
import { TrackLanes } from './TrackLanes';

/** Props of {@link TimelineView}. */
export interface TimelineViewProps {
  readonly model: TimelineModel;
  readonly api: TimelineViewportApi;
  readonly playhead: PlayheadControls;
  readonly inOut: InOutControls;
  readonly commands: TimelineCommands;
}

/**
 * The timeline body: ruler on top, track headers and lanes in a vertically scrolling area, the
 * overview strip at the bottom. Horizontal scrolling and zoom are virtual (wheel, minimap,
 * keyboard): only what is in view is rendered.
 */
export function TimelineView({ model, api, playhead, inOut, commands }: TimelineViewProps) {
  const root = useRef<HTMLDivElement>(null);
  const lanesId = useId();
  const trackHeights = useStore(timelineViewStore, (state) => state.trackHeights);
  const selectedIds = useSelectedItemIds();
  const points = useInOutPoints(model.sequence.id);
  const layout = layoutTracks(
    model.tracks,
    (track) => trackHeights[track.id] ?? DEFAULT_TRACK_HEIGHT,
  );
  useTimelineWheel({ root, layout, api });
  const span = inOutSpan(points, model.contentEnd);
  const snapCandidates = (): Flicks[] => [
    ...model.editPoints,
    ...model.markers.map((marker) => marker.time),
    ...[points.inPoint, points.outPoint].filter((point): point is Flicks => point !== null),
  ];
  return (
    <div ref={root} className="flex h-full min-h-0 flex-col">
      <div className="flex shrink-0">
        <div
          className="shrink-0 border-r border-b border-border bg-panel"
          style={{ width: TRACK_HEADER_WIDTH }}
        />
        <TimeRuler
          sequenceId={model.sequence.id}
          markers={model.markers}
          span={span}
          viewport={api.viewport}
          frameRate={model.sequence.format.frameRate}
          snapCandidates={snapCandidates}
          movePlayhead={(time) => {
            playhead.moveTo(time);
          }}
          inOut={inOut}
          onMarkerChange={(marker, patch) => {
            void commands.updateMarker(marker.id, patch);
          }}
          onMarkerRemove={(marker) => {
            void commands.removeMarker(marker.id);
          }}
          onMarkerGoTo={(time) => {
            playhead.moveTo(time, true);
          }}
        />
      </div>
      <div className="min-h-0 flex-1 overflow-x-hidden overflow-y-auto">
        <div className="flex min-h-full" style={{ height: layout.height }}>
          <TrackHeaderList model={model} layout={layout} commands={commands} />
          <TrackLanes
            id={lanesId}
            model={model}
            layout={layout}
            viewport={api.viewport}
            selected={new Set(selectedIds)}
            span={span}
          />
        </div>
      </div>
      <div className="flex shrink-0 border-t border-border">
        <div
          className="shrink-0 border-r border-border bg-panel"
          style={{ width: TRACK_HEADER_WIDTH }}
        />
        <Minimap model={model} layout={layout} api={api} lanesId={lanesId} />
      </div>
    </div>
  );
}
