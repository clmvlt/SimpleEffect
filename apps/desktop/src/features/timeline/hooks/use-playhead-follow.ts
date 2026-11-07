import type { Flicks, SequenceId } from '@se/protocol';
import { useEffect, useEffectEvent } from 'react';

import { playbackStatusStore } from '../../../shared/playback';
import { playheadStore, playheadTimeOf } from '../store/playhead-store';

/**
 * While a playback runs, scrolls the timeline by pages to keep the playhead in view (`page`
 * receives the new time and the playing direction). Follows the stores directly: no re-render
 * per frame.
 */
export function usePlayheadFollow(
  sequenceId: SequenceId,
  page: (time: Flicks, direction: 1 | -1) => void,
): void {
  const onMove = useEffectEvent((time: Flicks) => {
    const { rate } = playbackStatusStore.getState();
    if (rate !== 0) page(time, rate > 0 ? 1 : -1);
  });
  useEffect(
    () =>
      playheadStore.subscribe((state, previous) => {
        const time = playheadTimeOf(state, sequenceId);
        if (time !== playheadTimeOf(previous, sequenceId)) onMove(time);
      }),
    [sequenceId],
  );
}
