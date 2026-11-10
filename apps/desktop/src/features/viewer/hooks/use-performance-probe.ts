import type { Flicks, SequenceId } from '@se/protocol';
import { useEffect } from 'react';

import { registerViewerProbe } from '../../../shared/performance';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../../timeline';
import type { ViewerEngineController } from '../services/viewer-engine-controller';

/**
 * Registers the viewer for the performance measure: a seek goes through the playhead store like
 * a user's scrub, and is timed until the engine drew its frame.
 */
export function usePerformanceProbe(
  engine: ViewerEngineController,
  sequenceId: SequenceId,
  contentEnd: Flicks,
): void {
  useEffect(
    () =>
      registerViewerProbe({
        sequenceId,
        contentEnd,
        time: () => playheadTimeOf(playheadStore.getState(), sequenceId),
        async seek(time) {
          const start = performance.now();
          setPlayheadTime(sequenceId, time);
          await engine.settled();
          return performance.now() - start;
        },
        stats: () => engine.stats(),
      }),
    [engine, sequenceId, contentEnd],
  );
}
