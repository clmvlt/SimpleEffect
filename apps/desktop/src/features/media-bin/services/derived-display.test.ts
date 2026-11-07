import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { testAsset } from '../testing/assets';
import { audioFigures } from './analysis-format';
import { canGenerateProxy, derivedDisplay } from './derived-display';
import { IDLE_STATUS } from './job-status';

const pending = {
  relativePath: 'cache/media/k/mezzanine.mp4',
  status: { kind: 'pending' },
} as const;

describe('derivedDisplay', () => {
  it('follows the document state and the running job', () => {
    const asset = testAsset({ derived: { mezzanine: pending, proxy: null } });
    expect(derivedDisplay(asset, 'proxy', IDLE_STATUS)).toEqual({ kind: 'none' });
    expect(derivedDisplay(asset, 'mezzanine', IDLE_STATUS)).toEqual({ kind: 'queued' });
    const running = { ...IDLE_STATUS, running: [{ kind: 'mezzanine' as const, progress: 0.4 }] };
    expect(derivedDisplay(asset, 'mezzanine', running)).toEqual({ kind: 'running', progress: 0.4 });
    const failed = testAsset({
      derived: { mezzanine: { ...pending, status: { kind: 'failed', message: 'x' } }, proxy: null },
    });
    expect(derivedDisplay(failed, 'mezzanine', IDLE_STATUS)).toEqual({
      kind: 'failed',
      message: 'x',
    });
  });
});

describe('canGenerateProxy', () => {
  it('offers proxies for videos without one or whose proxy failed', () => {
    expect(canGenerateProxy(testAsset({}))).toBe(true);
    const proxy = {
      relativePath: 'cache/media/k/proxy-540.mp4',
      status: { kind: 'ready' },
    } as const;
    expect(canGenerateProxy(testAsset({ derived: { mezzanine: null, proxy } }))).toBe(false);
    const failed = { ...proxy, status: { kind: 'failed', message: 'x' } } as const;
    expect(canGenerateProxy(testAsset({ derived: { mezzanine: null, proxy: failed } }))).toBe(true);
    expect(canGenerateProxy(testAsset({ kind: 'audio' }))).toBe(false);
    expect(canGenerateProxy(testAsset({ missing: true }))).toBe(false);
  });
});

describe('audioFigures', () => {
  it('formats loudness, peak, range and silences with one decimal', () => {
    const figures = audioFigures(
      {
        integratedLufs: -16.54,
        loudnessRangeLu: null,
        truePeakDbtp: -1,
        silenceCount: 3,
        silenceDuration: (1.5 * 705_600_000) as Flicks,
      },
      'fr',
    );
    expect(figures).toEqual({
      loudness: '-16,5',
      truePeak: '-1,0',
      loudnessRange: null,
      silenceCount: 3,
      silenceSeconds: '1,5',
    });
  });
});
