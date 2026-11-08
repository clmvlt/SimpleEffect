import { useEffect } from 'react';

import { onPlayRequest } from '../../../shared/playback';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../../timeline';
import type { PlaybackSession } from '../services/playback-session';

/**
 * Plays the passages other features ask for (`requestPlayRange`): the playhead goes to the start,
 * playback starts, and pauses once it reaches the end (or when the user stops or seeks away).
 */
export function usePlayRequests(session: PlaybackSession): void {
  useEffect(() => {
    let stopWatching: (() => void) | null = null;
    const unsubscribe = onPlayRequest((request) => {
      stopWatching?.();
      setPlayheadTime(request.sequenceId, request.start);
      if (session.getState().rate === 0) session.togglePlay();
      stopWatching = playheadStore.subscribe((state) => {
        const time = playheadTimeOf(state, request.sequenceId);
        if (time >= request.end || time < request.start) {
          stopWatching?.();
          stopWatching = null;
          if (time >= request.end) session.pause();
        }
      });
    });
    return () => {
      stopWatching?.();
      unsubscribe();
    };
  }, [session]);
}
