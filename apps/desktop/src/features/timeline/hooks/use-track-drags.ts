import type { TrackEntity, TrackId } from '@se/protocol';
import { useRef, useState, type PointerEvent } from 'react';

import {
  dropIndex,
  dropLineTop,
  reorderedTrackIds,
  type TrackLayout,
} from '../services/track-layout';
import { setTrackHeights } from '../store/timeline-view-store';
import type { TimelineCommands } from './use-timeline-commands';
import type { TimelineModel } from './use-timeline-model';

interface Reorder {
  readonly track: TrackEntity;
  readonly listTop: number;
  readonly y: number;
}

/** The y of a pointer in the header list (which scrolls with the tracks). */
function listY(event: PointerEvent<HTMLElement>, listTop: number): number {
  return event.clientY - listTop;
}

/**
 * Pointer drags on track headers: the grip reorders a track among those of its kind (one
 * `reorderTracks` on release), the bottom edge resizes it (a machine preference, saved live).
 */
export function useTrackDrags(
  model: TimelineModel,
  layout: TrackLayout,
  commands: TimelineCommands,
) {
  const [reorder, setReorder] = useState<Reorder | null>(null);
  const resize = useRef<{ trackId: TrackId; startY: number; startHeight: number } | null>(null);
  const sectionOf = (track: TrackEntity) => (track.kind === 'video' ? layout.video : layout.audio);
  const target = (drag: Reorder) => dropIndex(sectionOf(drag.track), drag.track.id, drag.y);

  const grip = (track: TrackEntity) => ({
    onPointerDown: (event: PointerEvent<HTMLElement>) => {
      if (event.button !== 0) return;
      event.currentTarget.setPointerCapture(event.pointerId);
      const list = event.currentTarget.closest('[data-track-headers]');
      const listTop = list?.getBoundingClientRect().top ?? 0;
      setReorder({ track, listTop, y: listY(event, listTop) });
    },
    onPointerMove: (event: PointerEvent<HTMLElement>) => {
      if (reorder?.track.id === track.id)
        setReorder({ ...reorder, y: listY(event, reorder.listTop) });
    },
    onPointerUp: () => {
      if (reorder?.track.id !== track.id) return;
      const kindOf = (id: TrackId) => model.tracks.find((candidate) => candidate.id === id)?.kind;
      const order = reorderedTrackIds(model.sequence.trackIds, kindOf, track.id, target(reorder));
      if (order.some((id, index) => id !== model.sequence.trackIds[index])) {
        void commands.reorderTracks(model.sequence.id, order);
      }
      setReorder(null);
    },
    onPointerCancel: () => {
      setReorder(null);
    },
  });

  const resizeHandle = (track: TrackEntity, height: number) => ({
    onPointerDown: (event: PointerEvent<HTMLElement>) => {
      if (event.button !== 0) return;
      event.stopPropagation();
      event.currentTarget.setPointerCapture(event.pointerId);
      resize.current = { trackId: track.id, startY: event.clientY, startHeight: height };
    },
    onPointerMove: (event: PointerEvent<HTMLElement>) => {
      const current = resize.current;
      if (current?.trackId !== track.id) return;
      setTrackHeights([track.id], current.startHeight + event.clientY - current.startY);
    },
    onPointerUp: () => {
      resize.current = null;
    },
  });

  const dropLine =
    reorder === null
      ? null
      : dropLineTop(sectionOf(reorder.track), reorder.track.id, target(reorder));
  return { grip, resizeHandle, draggedTrackId: reorder?.track.id ?? null, dropLine };
}
