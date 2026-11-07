import type { PlaybackClock } from '../playback/playback-controller';

/** Room left for the first scheduling tick after an anchor (decoding usually done already). */
const SCHEDULING_MARGIN_SECONDS = 0.03;

/**
 * The master clock of playback (`docs/architecture.md` §8): the context time of the sample being
 * heard now — `getOutputTimestamp()` interpolated with `performance.now()` between audio
 * callbacks, never ahead of the render edge (`currentTime`), never going back. Pictures follow
 * what is heard, so they stay in sync whatever the output latency. It stops while the context is
 * suspended.
 */
export function audioContextClock(context: AudioContext): PlaybackClock {
  let last = 0;
  const now = (): number => {
    const stamp = context.getOutputTimestamp();
    const { contextTime, performanceTime } = stamp;
    let heard: number;
    if (contextTime !== undefined && performanceTime !== undefined && performanceTime > 0) {
      heard = contextTime + (performance.now() - performanceTime) / 1000;
    } else {
      heard = context.currentTime - (context.outputLatency || context.baseLatency || 0);
    }
    last = Math.max(last, Math.min(heard, context.currentTime));
    return last;
  };
  return {
    now,
    leadTime: () => Math.max(0, context.currentTime - now()) + SCHEDULING_MARGIN_SECONDS,
  };
}
