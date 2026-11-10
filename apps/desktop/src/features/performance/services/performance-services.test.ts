import type { PreviewStats } from '@se/engine/preview';
import type { Flicks } from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import type { ViewerProbe } from '../../../shared/performance';
import { budgetRows } from './budgets';
import {
  measurePerformance,
  scrubTimes,
  type MeasureDependencies,
  type MeasureStep,
} from './measure-performance';
import { summarize } from './statistics';

const SECOND = 705_600_000;

function stats(droppedFrames: number): PreviewStats {
  return {
    open: { videoFrames: 6, images: 0 },
    cachedPictures: 6,
    cachedBytes: 0,
    openSources: 1,
    readers: 1,
    queuedDecodes: 0,
    framesPerSecond: 30,
    droppedFrames,
    prefetchSeconds: 1,
  };
}

/** A clock the fakes advance: each ping takes 2 ms, each seek 40 ms (the last two scrub steps 150 ms). */
function fakes(probe: ViewerProbe | null) {
  let clock = 0;
  const steps: MeasureStep[] = [];
  const played: [number, number][] = [];
  const dependencies: MeasureDependencies = {
    pingCore: () => {
      clock += 2;
      return Promise.resolve();
    },
    probe,
    playRange: (_sequence, start, end) => {
      played.push([start, end]);
    },
    recordFrames: () => () => [16, 17, 16, 70, 16],
    wait: () => Promise.resolve(),
    now: () => clock,
    heapBytes: () => 120 * 1_048_576,
    onStep: (step) => {
      steps.push(step);
    },
  };
  return { dependencies, steps, played };
}

function probe(contentEnd: number) {
  const seeks: number[] = [];
  const viewer: ViewerProbe = {
    sequenceId: 'seq',
    contentEnd: contentEnd as Flicks,
    time: () => (5 * SECOND) as Flicks,
    seek: (time) => {
      seeks.push(time);
      return Promise.resolve(seeks.length >= 19 && seeks.length <= 20 ? 150 : 40);
    },
    stats: () => stats(2),
  };
  return { viewer, seeks };
}

describe('summarize', () => {
  it('gives the median, the 95th percentile and the maximum', () => {
    const values = Array.from({ length: 100 }, (_, index) => index + 1);
    expect(summarize(values)).toEqual({ count: 100, median: 50, p95: 95, max: 100 });
    expect(summarize([])).toEqual({ count: 0, median: 0, p95: 0, max: 0 });
  });
});

describe('scrubTimes', () => {
  it('spreads jumping times inside the content', () => {
    const times = scrubTimes((60 * SECOND) as Flicks, 20);
    expect(new Set(times).size).toBe(20);
    expect(times.every((time) => time >= 0 && time < 60 * SECOND)).toBe(true);
    expect(times.some((time, index) => index > 0 && time < (times[index - 1] ?? 0))).toBe(true);
  });
});

describe('measurePerformance', () => {
  it('measures only the core without a viewer', async () => {
    const { dependencies, steps } = fakes(null);
    const report = await measurePerformance(dependencies);
    expect(steps).toEqual(['core']);
    expect(report.core).toMatchObject({ count: 30, median: 2, p95: 2 });
    expect(report.scrub).toBeNull();
    expect(report.heapMegabytes).toBe(120);
    expect(budgetRows(report).map((row) => row.status)).toEqual([
      'within',
      'skipped',
      'skipped',
      'skipped',
      'skipped',
    ]);
  });

  it('scrubs, plays three seconds and puts the playhead back', async () => {
    const { viewer, seeks } = probe(60 * SECOND);
    const { dependencies, steps, played } = fakes(viewer);
    const report = await measurePerformance(dependencies);
    expect(steps).toEqual(['core', 'scrub', 'playback']);
    expect(report.scrub).toMatchObject({ count: 20, median: 40, max: 150 });
    expect(played).toEqual([[15 * SECOND, 18 * SECOND]]);
    expect(seeks.at(-1)).toBe(5 * SECOND);
    expect(report.longFrames).toBe(1);
    expect(report.playback).toEqual({ droppedFrames: 2, framesPerSecond: 30 });
    expect(report.openVideoFrames).toBe(6);
    const rows = budgetRows(report);
    expect(rows.find((row) => row.id === 'scrub')?.status).toBe('over');
    expect(rows.find((row) => row.id === 'droppedFrames')?.status).toBe('over');
    expect(rows.find((row) => row.id === 'longFrames')?.status).toBe('over');
  });

  it('plays from the start of a short sequence', async () => {
    const { viewer } = probe(2 * SECOND);
    const { dependencies, played } = fakes(viewer);
    await measurePerformance(dependencies);
    expect(played).toEqual([[0, 3 * SECOND]]);
  });

  it('reports a failing core as a rejection', async () => {
    const { dependencies } = fakes(null);
    dependencies.pingCore = vi.fn(() => Promise.reject(new Error('down')));
    await expect(measurePerformance(dependencies)).rejects.toThrow('down');
  });
});
