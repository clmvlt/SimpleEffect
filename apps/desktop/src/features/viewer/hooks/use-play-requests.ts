import { useEffect } from 'react';

import { onPlayRequest } from '../../../shared/playback';
import { FLICKS_PER_SECOND } from '../../../shared/time';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../../timeline';
import type { PlaybackSession } from '../services/playback-session';

/** Extra time before the timer pauses a passage whose end the playhead did not report. */
const TIMER_SLACK_MS = 250;

/**
 * Plays the passages other features ask for (`requestPlayRange`): the playhead goes to the start,
 * playback starts, and pauses once it reaches the end (or stops watching when the user seeks
 * away). A timer pauses it too, after the passage's duration: a hidden window throttles the
 * playhead's frames while its sound keeps playing.
 */
export function usePlayRequests(session: PlaybackSession): void {
  useEffect(() => {
    let stopWatching: (() => void) | null = null;
    const unsubscribe = onPlayRequest((request) => {
      stopWatching?.();
      setPlayheadTime(request.sequenceId, request.start);
      if (session.getState().rate === 0) session.togglePlay();
      const finish = () => {
        stopWatching?.();
        stopWatching = null;
        session.pause();
      };
      const duration = ((request.end - request.start) / FLICKS_PER_SECOND) * 1000;
      const timer = setTimeout(finish, duration + TIMER_SLACK_MS);
      const unwatch = playheadStore.subscribe((state) => {
        const time = playheadTimeOf(state, request.sequenceId);
        if (time >= request.end) finish();
        else if (time < request.start) stopWatching?.();
      });
      stopWatching = () => {
        clearTimeout(timer);
        unwatch();
      };
    });
    return () => {
      stopWatching?.();
      unsubscribe();
    };
  }, [session]);
}
