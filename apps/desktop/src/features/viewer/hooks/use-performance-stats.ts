import type { AudioEngineStats } from '@se/engine/audio';
import type { PreviewStats } from '@se/engine/preview';
import { useEffect, useState } from 'react';

import type { PlaybackSession } from '../services/playback-session';
import type { ViewerEngineController } from '../services/viewer-engine-controller';

/** Counters shown by the performance HUD. */
export interface PerformanceStats {
  readonly picture: PreviewStats | null;
  readonly audio: AudioEngineStats | null;
}

const REFRESH_MS = 250;

/** Picture and audio counters, polled four times per second while the HUD shows. */
export function usePerformanceStats(
  engine: ViewerEngineController,
  session: PlaybackSession,
): PerformanceStats {
  const read = (): PerformanceStats => ({
    picture: engine.stats(),
    audio: session.audioStats(),
  });
  const [stats, setStats] = useState<PerformanceStats>(read);
  useEffect(() => {
    const timer = setInterval(() => {
      setStats({ picture: engine.stats(), audio: session.audioStats() });
    }, REFRESH_MS);
    return () => {
      clearInterval(timer);
    };
  }, [engine, session]);
  return stats;
}
