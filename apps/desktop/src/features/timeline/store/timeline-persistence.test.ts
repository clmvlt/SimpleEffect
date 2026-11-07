import { afterEach, describe, expect, it, vi } from 'vitest';

import type { KeyValueStorage } from '../../../shared/storage';
import { seconds } from '../testing/fixtures';
import { playheadStore, setPlayheadTime } from './playhead-store';
import { connectTimelinePersistence, parseSavedView } from './timeline-persistence';
import {
  setActiveSequence,
  setSnapping,
  setTrackHeights,
  timelineViewStore,
  updateSequenceView,
} from './timeline-view-store';

function memoryStorage(): KeyValueStorage & { entries: Map<string, string> } {
  const entries = new Map<string, string>();
  return {
    entries,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => {
      entries.set(key, value);
    },
  };
}

afterEach(() => {
  vi.useRealTimers();
});

describe('timeline persistence', () => {
  it('saves the view of a project and restores it when the project is shown again', () => {
    vi.useFakeTimers();
    const storage = memoryStorage();
    const disconnect = connectTimelinePersistence('project-1', storage);
    setActiveSequence('sequence-2');
    updateSequenceView('sequence-2', {
      pixelsPerSecond: 80,
      scrollTime: seconds(12),
      inPoint: seconds(2),
      outPoint: seconds(5),
    });
    setPlayheadTime('sequence-2', seconds(3));
    setTrackHeights(['track-1'], 90);
    setSnapping(false);
    vi.advanceTimersByTime(1000);
    disconnect();

    connectTimelinePersistence('project-2', storage)();
    expect(timelineViewStore.getState().activeSequenceId).toBeNull();
    expect(playheadStore.getState().times).toEqual({});

    const reconnect = connectTimelinePersistence('project-1', storage);
    const view = timelineViewStore.getState();
    expect(view.activeSequenceId).toBe('sequence-2');
    expect(view.sequences['sequence-2']).toEqual({
      pixelsPerSecond: 80,
      scrollTime: seconds(12),
      inPoint: seconds(2),
      outPoint: seconds(5),
    });
    expect(view.trackHeights).toEqual({ 'track-1': 90 });
    expect(view.snapping).toBe(false);
    expect(playheadStore.getState().times).toEqual({ 'sequence-2': seconds(3) });
    reconnect();
  });

  it('saves pending changes when it disconnects', () => {
    const storage = memoryStorage();
    const disconnect = connectTimelinePersistence('project-3', storage);
    setTrackHeights(['track-9'], 40);
    disconnect();
    expect(parseSavedView(storage.getItem('timeline.v1.project.project-3'))?.trackHeights).toEqual({
      'track-9': 40,
    });
  });

  it('drops malformed saved values', () => {
    expect(parseSavedView('not json')).toBeNull();
    expect(parseSavedView(null)).toBeNull();
    const parsed = parseSavedView(
      JSON.stringify({
        activeSequenceId: 4,
        sequences: {
          s: { pixelsPerSecond: -1, scrollTime: 1.5, inPoint: 'x', playhead: 99 },
          t: 3,
        },
        trackHeights: { a: 1000, b: 'tall' },
      }),
    );
    expect(parsed).toEqual({
      activeSequenceId: null,
      sequences: {
        s: { pixelsPerSecond: null, scrollTime: 0, inPoint: null, outPoint: null, playhead: 99 },
      },
      trackHeights: { a: 200 },
    });
  });
});
