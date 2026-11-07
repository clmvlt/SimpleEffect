import type { Flicks, ProjectId, SequenceId, TrackId } from '@se/protocol';

import type { KeyValueStorage } from '../../../shared/storage';
import { isFlicks } from '../../../shared/time';
import { clampTrackHeight } from '../services/track-height';
import { playheadStore } from './playhead-store';
import { NEW_SEQUENCE_VIEW, timelineViewStore, type SequenceView } from './timeline-view-store';

/**
 * Saves the timeline view of each project (active sequence, zoom, scroll, in/out, playhead,
 * track heights) and the snapping preference in the machine's UI preferences, never in the
 * project: reopening a project on this machine shows it as it was left.
 */

/** What is saved for one project. */
export interface SavedTimelineView {
  readonly activeSequenceId: SequenceId | null;
  readonly sequences: Readonly<Record<SequenceId, SequenceView & { readonly playhead: Flicks }>>;
  readonly trackHeights: Readonly<Record<TrackId, number>>;
}

const projectKey = (projectId: ProjectId) => `timeline.v1.project.${projectId}`;
const SNAPPING_KEY = 'timeline.v1.snapping';
const SAVE_DELAY_MS = 400;

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === 'object' && value !== null && !Array.isArray(value);

const flicksOr = (value: unknown, fallback: Flicks | null): Flicks | null =>
  typeof value === 'number' && isFlicks(value) && value >= 0 ? value : fallback;

function parseSequenceView(value: unknown): (SequenceView & { playhead: Flicks }) | null {
  if (!isRecord(value)) return null;
  const zoom = value['pixelsPerSecond'];
  return {
    pixelsPerSecond: typeof zoom === 'number' && Number.isFinite(zoom) && zoom > 0 ? zoom : null,
    scrollTime:
      flicksOr(value['scrollTime'], NEW_SEQUENCE_VIEW.scrollTime) ?? NEW_SEQUENCE_VIEW.scrollTime,
    inPoint: flicksOr(value['inPoint'], null),
    outPoint: flicksOr(value['outPoint'], null),
    playhead: flicksOr(value['playhead'], 0 as Flicks) ?? (0 as Flicks),
  };
}

/** The saved view of a project read back; anything malformed is dropped. */
export function parseSavedView(text: string | null): SavedTimelineView | null {
  if (text === null) return null;
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  if (!isRecord(value)) return null;
  const sequences = isRecord(value['sequences']) ? value['sequences'] : {};
  const heights = isRecord(value['trackHeights']) ? value['trackHeights'] : {};
  const active = value['activeSequenceId'];
  return {
    activeSequenceId: typeof active === 'string' ? active : null,
    sequences: Object.fromEntries(
      Object.entries(sequences).flatMap(([id, view]) => {
        const parsed = parseSequenceView(view);
        return parsed === null ? [] : [[id, parsed]];
      }),
    ),
    trackHeights: Object.fromEntries(
      Object.entries(heights).flatMap(([id, height]) =>
        typeof height === 'number' ? [[id, clampTrackHeight(height)]] : [],
      ),
    ),
  };
}

/** The view of the current state to save. */
function currentView(): SavedTimelineView {
  const view = timelineViewStore.getState();
  const { times } = playheadStore.getState();
  const ids = new Set([...Object.keys(view.sequences), ...Object.keys(times)]);
  return {
    activeSequenceId: view.activeSequenceId,
    sequences: Object.fromEntries(
      [...ids].map((id) => [
        id,
        { ...(view.sequences[id] ?? NEW_SEQUENCE_VIEW), playhead: times[id] ?? (0 as Flicks) },
      ]),
    ),
    trackHeights: view.trackHeights,
  };
}

/** Puts the saved view of `projectId` (or a fresh one) into the stores. */
function restore(projectId: ProjectId, saved: SavedTimelineView | null, snapping: boolean): void {
  const sequences = saved?.sequences ?? {};
  timelineViewStore.setState({
    projectId,
    activeSequenceId: saved?.activeSequenceId ?? null,
    sequences: Object.fromEntries(
      Object.entries(sequences).map(([id, view]) => [
        id,
        {
          pixelsPerSecond: view.pixelsPerSecond,
          scrollTime: view.scrollTime,
          inPoint: view.inPoint,
          outPoint: view.outPoint,
        },
      ]),
    ),
    trackHeights: saved?.trackHeights ?? {},
    snapping,
  });
  playheadStore.setState({
    times: Object.fromEntries(Object.entries(sequences).map(([id, view]) => [id, view.playhead])),
  });
}

/**
 * Loads the saved view of `projectId` into the stores, then saves changes (debounced) until the
 * returned function is called, which saves one last time.
 */
export function connectTimelinePersistence(
  projectId: ProjectId,
  storage: KeyValueStorage,
): () => void {
  restore(
    projectId,
    parseSavedView(storage.getItem(projectKey(projectId))),
    storage.getItem(SNAPPING_KEY) !== 'false',
  );
  let timer: ReturnType<typeof setTimeout> | undefined;
  const save = () => {
    clearTimeout(timer);
    timer = undefined;
    storage.setItem(projectKey(projectId), JSON.stringify(currentView()));
    storage.setItem(SNAPPING_KEY, String(timelineViewStore.getState().snapping));
  };
  const schedule = () => {
    clearTimeout(timer);
    timer = setTimeout(save, SAVE_DELAY_MS);
  };
  const unsubscribeView = timelineViewStore.subscribe(schedule);
  const unsubscribePlayhead = playheadStore.subscribe(schedule);
  return () => {
    unsubscribeView();
    unsubscribePlayhead();
    if (timer !== undefined) save();
  };
}
