import type { Flicks, SequenceId } from '@se/protocol';

import type { ViewerProbe } from '../../../shared/performance';
import { FLICKS_PER_SECOND } from '../../../shared/time';
import { BUDGETS } from './budgets';
import { summarize, type DurationSummary } from './statistics';

/** Commands timed against se-core. */
const CORE_ROUND_TRIPS = 30;
/** Scrub steps, at spread times of the content. */
const SCRUB_STEPS = 20;
/** Seconds played at 1× to count dropped frames. */
const PLAYBACK_SECONDS = 3;
/**
 * A frame not drawn after this long means the viewer is stalled (a hidden window draws nothing:
 * its animation frames stop): the measure stops instead of waiting forever.
 */
export const SEEK_TIMEOUT_MS = 10_000;

/** Why a measure could not run to its end. */
export class MeasureStoppedError extends Error {
  constructor(readonly reason: 'hiddenWindow' | 'viewerStalled') {
    super(`performance measure stopped: ${reason}`);
    this.name = 'MeasureStoppedError';
  }
}

/** The step running, for the progress line. */
export type MeasureStep = 'core' | 'scrub' | 'playback';

/** What the measure needs from the app (injected: tests use fakes). */
export interface MeasureDependencies {
  /** One round trip to se-core (`ping`). */
  pingCore(): Promise<void>;
  /** The viewer showing a sequence, or `null`: viewer steps are skipped. */
  readonly probe: ViewerProbe | null;
  /** Plays `[start, end)` of the sequence at 1× (`requestPlayRange`). */
  playRange(sequenceId: SequenceId, start: Flicks, end: Flicks): void;
  /** Starts recording main-thread frame intervals; the returned function stops and gives them. */
  recordFrames(): () => number[];
  wait(milliseconds: number): Promise<void>;
  now(): number;
  /** JavaScript heap in use, in bytes, when the platform tells (Chromium). */
  heapBytes(): number | null;
  /** Whether the window is shown: a hidden one draws no frame, so nothing can be measured. */
  windowVisible(): boolean;
  /** Called when a step starts. */
  readonly onStep?: ((step: MeasureStep) => void) | undefined;
}

/** Playback counters after a few seconds at 1×. */
export interface PlaybackResult {
  readonly droppedFrames: number;
  readonly framesPerSecond: number;
}

/** What one measure found. */
export interface PerformanceReport {
  readonly core: DurationSummary;
  readonly scrub: DurationSummary | null;
  /** Main-thread frame intervals while the viewer scrubbed and played. */
  readonly frames: DurationSummary | null;
  /** Intervals over {@link BUDGETS.longFrameMs}. */
  readonly longFrames: number | null;
  readonly playback: PlaybackResult | null;
  /** `VideoFrame`s held by the decoder pool after the measure (its cache included). */
  readonly openVideoFrames: number | null;
  readonly heapMegabytes: number | null;
}

async function measureCore(dependencies: MeasureDependencies): Promise<DurationSummary> {
  const durations: number[] = [];
  for (let index = 0; index < CORE_ROUND_TRIPS; index += 1) {
    const start = dependencies.now();
    await dependencies.pingCore();
    durations.push(dependencies.now() - start);
  }
  return summarize(durations);
}

/** `count` times spread over the content in a fixed, jumping order (golden-ratio steps). */
export function scrubTimes(contentEnd: Flicks, count: number): Flicks[] {
  return Array.from({ length: count }, (_, index) => {
    const fraction = (index * 0.618_033_988_75) % 1;
    return Math.floor(fraction * contentEnd) as Flicks;
  });
}

/** Seeks `probe` to `time`, failing when the frame is not drawn within {@link SEEK_TIMEOUT_MS}. */
async function timedSeek(
  dependencies: MeasureDependencies,
  probe: ViewerProbe,
  time: Flicks,
): Promise<number> {
  const stalled = dependencies.wait(SEEK_TIMEOUT_MS).then(() => null);
  const duration = await Promise.race([probe.seek(time), stalled]);
  if (duration !== null) return duration;
  throw new MeasureStoppedError(dependencies.windowVisible() ? 'viewerStalled' : 'hiddenWindow');
}

async function measureScrub(
  dependencies: MeasureDependencies,
  probe: ViewerProbe,
): Promise<DurationSummary> {
  const durations: number[] = [];
  for (const time of scrubTimes(probe.contentEnd, SCRUB_STEPS)) {
    durations.push(await timedSeek(dependencies, probe, time));
  }
  return summarize(durations);
}

async function measurePlayback(
  dependencies: MeasureDependencies,
  probe: ViewerProbe,
): Promise<PlaybackResult | null> {
  const length = PLAYBACK_SECONDS * FLICKS_PER_SECOND;
  const start = Math.max(0, Math.min(Math.floor(probe.contentEnd / 4), probe.contentEnd - length));
  await timedSeek(dependencies, probe, start as Flicks);
  dependencies.playRange(probe.sequenceId, start as Flicks, (start + length) as Flicks);
  // Read the counters just before the range ends: the engine resets them at the next playback.
  await dependencies.wait(PLAYBACK_SECONDS * 1000 - 250);
  const stats = probe.stats();
  await dependencies.wait(500);
  return stats === null
    ? null
    : { droppedFrames: stats.droppedFrames, framesPerSecond: stats.framesPerSecond };
}

function memoryOf(dependencies: MeasureDependencies, probe: ViewerProbe | null) {
  const heap = dependencies.heapBytes();
  return {
    openVideoFrames: probe?.stats()?.open.videoFrames ?? null,
    heapMegabytes: heap === null ? null : Math.round(heap / 1_048_576),
  };
}

/**
 * Measures the budgets of `docs/performance.md` in the running app: core round trips, then (with
 * a sequence in the viewer) scrub latency and a few seconds of playback while the main thread's
 * frames are recorded. The playhead goes back where it was.
 */
export async function measurePerformance(
  dependencies: MeasureDependencies,
): Promise<PerformanceReport> {
  if (!dependencies.windowVisible()) throw new MeasureStoppedError('hiddenWindow');
  dependencies.onStep?.('core');
  const core = await measureCore(dependencies);
  const { probe } = dependencies;
  if (probe === null || probe.contentEnd <= 0) {
    return {
      core,
      scrub: null,
      frames: null,
      longFrames: null,
      playback: null,
      ...memoryOf(dependencies, null),
    };
  }
  return { core, ...(await measureViewer(dependencies, probe)) };
}

/** Scrub and playback with main-thread frames recorded; the playhead goes back afterwards. */
async function measureViewer(dependencies: MeasureDependencies, probe: ViewerProbe) {
  const startTime = probe.time();
  const stopRecording = dependencies.recordFrames();
  dependencies.onStep?.('scrub');
  let scrub: DurationSummary;
  let playback: PlaybackResult | null;
  let intervals: number[];
  try {
    scrub = await measureScrub(dependencies, probe);
    dependencies.onStep?.('playback');
    playback = await measurePlayback(dependencies, probe);
  } finally {
    intervals = stopRecording();
  }
  await timedSeek(dependencies, probe, startTime);
  return {
    scrub,
    frames: summarize(intervals),
    longFrames: intervals.filter((interval) => interval > BUDGETS.longFrameMs).length,
    playback,
    ...memoryOf(dependencies, probe),
  };
}
