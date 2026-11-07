import type { Asset, DerivedFile, StreamDecodability } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { isPlayable, playbackSource } from './index';

const MEZZANINE_PATH = 'cache/media/k/mezzanine.mp4';

function mezzanine(status: DerivedFile['status']['kind']): DerivedFile {
  const statuses = {
    pending: { kind: 'pending' },
    ready: { kind: 'ready' },
    failed: { kind: 'failed', message: 'ffmpeg failed' },
  } as const;
  return { relativePath: MEZZANINE_PATH, status: statuses[status] };
}

function asset(
  video: StreamDecodability,
  audio: StreamDecodability,
  derivedMezzanine: DerivedFile | null = null,
  missing = false,
): Asset {
  return {
    id: '0193e7d2-0000-7000-8000-000000000001',
    originalPath: '/rushes/clip.mov',
    relativeHint: null,
    kind: 'video',
    mediaInfo: null,
    fingerprint: null,
    derived: { mezzanine: derivedMezzanine, proxy: null },
    analysis: { audio: null, scenes: null },
    decodability: { video, audio },
    missing,
  };
}

describe('playbackSource', () => {
  it('plays decodable originals', () => {
    expect(playbackSource(asset('decodable', 'decodable'))).toEqual({ kind: 'original' });
    expect(playbackSource(asset('decodable', 'noStream'))).toEqual({ kind: 'original' });
  });

  it('waits for the decodability check and for pending mezzanines', () => {
    expect(playbackSource(asset('unknown', 'decodable'))).toEqual({ kind: 'preparing' });
    expect(playbackSource(asset('undecodable', 'decodable'))).toEqual({ kind: 'preparing' });
    const layout = asset('decodable', 'decodable', mezzanine('pending'));
    expect(playbackSource(layout)).toEqual({ kind: 'preparing' });
    expect(isPlayable(layout)).toBe(false);
  });

  it('plays a ready mezzanine instead of the original', () => {
    const prores = asset('undecodable', 'decodable', mezzanine('ready'));
    expect(playbackSource(prores)).toEqual({ kind: 'mezzanine', relativePath: MEZZANINE_PATH });
    expect(isPlayable(prores)).toBe(true);
  });

  it('reports missing originals and failed mezzanines of undecodable media', () => {
    expect(playbackSource(asset('decodable', 'decodable', null, true))).toEqual({
      kind: 'unavailable',
      reason: 'missing',
    });
    expect(playbackSource(asset('undecodable', 'decodable', mezzanine('failed')))).toEqual({
      kind: 'unavailable',
      reason: 'mezzanineFailed',
    });
    expect(playbackSource(asset('decodable', 'decodable', mezzanine('failed')))).toEqual({
      kind: 'original',
    });
  });
});
