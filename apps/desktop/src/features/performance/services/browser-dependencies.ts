import type { CoreClient } from '@se/protocol';

import { currentViewerProbe } from '../../../shared/performance';
import { requestPlayRange } from '../../../shared/playback';
import type { MeasureDependencies, MeasureStep } from './measure-performance';

/** Chromium's non-standard heap counters (`performance.memory`). */
interface ChromiumPerformance {
  readonly memory?: { readonly usedJSHeapSize: number };
}

/** Records the intervals between animation frames until the returned function is called. */
function recordAnimationFrames(): () => number[] {
  const intervals: number[] = [];
  let previous: number | null = null;
  let handle = requestAnimationFrame(function tick(now) {
    if (previous !== null) intervals.push(now - previous);
    previous = now;
    handle = requestAnimationFrame(tick);
  });
  return () => {
    cancelAnimationFrame(handle);
    return intervals;
  };
}

/** The measure's dependencies in the window: the real core, viewer and animation frames. */
export function browserDependencies(
  client: Pick<CoreClient, 'send'>,
  onStep: (step: MeasureStep) => void,
): MeasureDependencies {
  return {
    pingCore: async () => {
      await client.send({ kind: 'ping' });
    },
    probe: currentViewerProbe(),
    playRange: (sequenceId, start, end) => {
      requestPlayRange({ sequenceId, start, end });
    },
    recordFrames: recordAnimationFrames,
    wait: (milliseconds) =>
      new Promise((resolve) => {
        setTimeout(resolve, milliseconds);
      }),
    now: () => performance.now(),
    heapBytes: () => (performance as ChromiumPerformance).memory?.usedJSHeapSize ?? null,
    windowVisible: () => document.visibilityState === 'visible',
    onStep,
  };
}
