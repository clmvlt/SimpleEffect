import type {
  Asset,
  Flicks,
  ItemEntity,
  MarkerEntity,
  MotionTemplate,
  SequenceEntity,
  TrackEntity,
} from '@se/protocol';
import { useStore } from 'zustand';

import { useProjectSelector } from '../../../shared/project-store';
import { contentEnd, editPoints } from '../services/edit-points';
import { timelineViewStore } from '../store/timeline-view-store';

/** What the timeline shows of the open project, derived from the read-only mirror. */
export interface TimelineModel {
  readonly projectId: string;
  /** Every sequence, in project order (the tabs). */
  readonly sequences: readonly SequenceEntity[];
  /** The sequence shown. */
  readonly sequence: SequenceEntity;
  /** Its tracks in document order. */
  readonly tracks: readonly TrackEntity[];
  /** Items of each track, sorted by start. */
  readonly itemsByTrack: ReadonlyMap<string, readonly ItemEntity[]>;
  /** Every item of the sequence. */
  readonly items: readonly ItemEntity[];
  /** Items of the project by id (link groups). */
  readonly itemsById: Readonly<Record<string, ItemEntity>>;
  readonly markers: readonly MarkerEntity[];
  readonly assets: Readonly<Record<string, Asset>>;
  readonly templates: readonly MotionTemplate[];
  /** End of the last item. */
  readonly contentEnd: Flicks;
  /** Sequence start and every item edge, sorted. */
  readonly editPoints: readonly Flicks[];
}

const isDefined = <T>(value: T | undefined): value is T => value !== undefined;

/**
 * The timeline model of the open project's active sequence (the one chosen in the tabs, or the
 * first), or `null` while no project with a sequence is loaded.
 */
export function useTimelineModel(): TimelineModel | null {
  const mirror = useProjectSelector((state) => state.mirror);
  const activeSequenceId = useStore(timelineViewStore, (state) => state.activeSequenceId);
  if (mirror === null) return null;
  const sequences = mirror.sequenceOrder.map((id) => mirror.sequences[id]).filter(isDefined);
  const sequence =
    (activeSequenceId === null ? undefined : mirror.sequences[activeSequenceId]) ?? sequences[0];
  if (sequence === undefined) return null;
  const tracks = sequence.trackIds.map((id) => mirror.tracks[id]).filter(isDefined);
  const itemsByTrack = new Map(
    tracks.map((track) => [
      track.id,
      track.itemIds.map((id) => mirror.items[id]).filter(isDefined),
    ]),
  );
  const items = [...itemsByTrack.values()].flat();
  return {
    projectId: mirror.projectId,
    sequences,
    sequence,
    tracks,
    itemsByTrack,
    items,
    itemsById: mirror.items,
    markers: sequence.markerIds.map((id) => mirror.markers[id]).filter(isDefined),
    assets: mirror.assets,
    templates: mirror.project.templates,
    contentEnd: contentEnd(items),
    editPoints: editPoints(items),
  };
}
