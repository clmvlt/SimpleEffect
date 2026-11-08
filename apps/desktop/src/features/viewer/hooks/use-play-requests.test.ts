/**
 * @vitest-environment jsdom
 */
import type { Flicks } from '@se/protocol';
import { renderHook } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { requestPlayRange } from '../../../shared/playback';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../../timeline';
import type { PlaybackSession } from '../services/playback-session';
import { usePlayRequests } from './use-play-requests';

const seconds = (value: number) => Math.round(value * 705_600_000) as Flicks;

function fakeSession(rate = 0) {
  const session = { rate, togglePlay: vi.fn(), pause: vi.fn() };
  const asSession = {
    getState: () => ({ rate: session.rate }),
    togglePlay: session.togglePlay,
    pause: session.pause,
  } as unknown as PlaybackSession;
  return { session, asSession };
}

afterEach(() => {
  vi.useRealTimers();
});

describe('usePlayRequests', () => {
  it('plays from the start and pauses when the playhead reaches the end', () => {
    const { session, asSession } = fakeSession();
    renderHook(() => {
      usePlayRequests(asSession);
    });
    requestPlayRange({ sequenceId: 'sequence-1', start: seconds(2), end: seconds(3) });
    expect(playheadTimeOf(playheadStore.getState(), 'sequence-1')).toBe(seconds(2));
    expect(session.togglePlay).toHaveBeenCalledTimes(1);
    setPlayheadTime('sequence-1', seconds(2.5));
    expect(session.pause).not.toHaveBeenCalled();
    setPlayheadTime('sequence-1', seconds(3));
    expect(session.pause).toHaveBeenCalledTimes(1);
  });

  it('pauses after the passage’s duration when the playhead stays still (hidden window)', () => {
    vi.useFakeTimers();
    const { session, asSession } = fakeSession(1);
    renderHook(() => {
      usePlayRequests(asSession);
    });
    requestPlayRange({ sequenceId: 'sequence-1', start: seconds(5), end: seconds(6) });
    expect(session.togglePlay).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1300);
    expect(session.pause).toHaveBeenCalledTimes(1);
  });

  it('lets the user take over when they seek before the passage', () => {
    vi.useFakeTimers();
    const { session, asSession } = fakeSession();
    renderHook(() => {
      usePlayRequests(asSession);
    });
    requestPlayRange({ sequenceId: 'sequence-1', start: seconds(5), end: seconds(6) });
    setPlayheadTime('sequence-1', seconds(1));
    vi.advanceTimersByTime(5000);
    expect(session.pause).not.toHaveBeenCalled();
  });
});
