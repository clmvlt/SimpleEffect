import { describe, expect, it } from 'vitest';

import { settle } from '../testing/fake-decoding';
import { assembleChunk, type AudioChunk } from './audio-chunk';
import { AudioChunkPool } from './audio-chunk-pool';
import type { AudioChunkReader } from './audio-source';
import { LevelAccumulator } from './meter/level-math';
import { dbToGain, gainToDb, stripSettings, type MixTrack } from './mix';

/** A minimal `AudioBuffer` for Node (no Web Audio). */
class FakeAudioBuffer {
  readonly channels: Float32Array[];
  constructor(
    readonly numberOfChannels: number,
    readonly length: number,
    readonly sampleRate: number,
  ) {
    this.channels = Array.from({ length: numberOfChannels }, () => new Float32Array(length));
  }
  get duration(): number {
    return this.length / this.sampleRate;
  }
  getChannelData(channel: number): Float32Array {
    const data = this.channels[channel];
    if (data === undefined) throw new Error(`no channel ${String(channel)}`);
    return data;
  }
  copyToChannel(source: Float32Array, channel: number, offset = 0): void {
    this.getChannelData(channel).set(source, offset);
  }
}

const createBuffer = (channels: number, length: number, rate: number) =>
  new FakeAudioBuffer(channels, length, rate) as unknown as AudioBuffer;

/** A mono buffer of `length` samples whose values are their index in the file plus one. */
function rampBuffer(first: number, length: number, rate = 100): AudioBuffer {
  const buffer = new FakeAudioBuffer(1, length, rate);
  buffer.getChannelData(0).forEach((_, index, data) => (data[index] = first + index + 1));
  return buffer as unknown as AudioBuffer;
}

const track = (id: string, patch: Partial<MixTrack> = {}): MixTrack => ({
  id,
  kind: 'audio',
  muted: false,
  solo: false,
  volumeDb: 0,
  pan: 0,
  ...patch,
});

describe('stripSettings', () => {
  it('applies mute, solo and volume', () => {
    const settings = stripSettings([
      track('a1', { volumeDb: -6 }),
      track('a2', { muted: true }),
      track('v1', { kind: 'video' }),
    ]);
    expect(settings.get('a1')?.gain).toBeCloseTo(0.501, 3);
    expect(settings.get('a2')?.gain).toBe(0);
    expect(settings.has('v1')).toBe(false);
    const solo = stripSettings([track('a1', { solo: true, pan: 3 }), track('a2')]);
    expect(solo.get('a1')).toEqual({ gain: 1, pan: 1 });
    expect(solo.get('a2')?.gain).toBe(0);
  });

  it('converts decibels both ways', () => {
    expect(dbToGain(0)).toBe(1);
    expect(gainToDb(dbToGain(-12))).toBeCloseTo(-12, 9);
    expect(gainToDb(0)).toBe(Number.NEGATIVE_INFINITY);
  });
});

describe('assembleChunk', () => {
  it('copies buffers at whole-sample positions so chunks join exactly', () => {
    // 100 Hz, origin 0.05 s, buffers of 30 samples covering the file from its first sample.
    const buffers = [0, 30, 60, 90, 120].map((first) => ({
      buffer: rampBuffer(first, 30),
      timestamp: 0.05 + first / 100,
    }));
    const zero = assembleChunk(buffers, 0, 0.05, createBuffer);
    const one = assembleChunk(buffers, 1, 0.05, createBuffer);
    expect(zero?.buffer.length).toBe(100);
    expect(zero?.buffer.getChannelData(0)[0]).toBe(1);
    expect(zero?.buffer.getChannelData(0)[99]).toBe(100);
    expect(one?.start).toBe(1);
    expect(one?.buffer.length).toBe(50);
    expect(one?.buffer.getChannelData(0)[0]).toBe(101);
  });

  it('returns null without samples in the chunk', () => {
    expect(assembleChunk([], 0, 0, createBuffer)).toBeNull();
    const early = [{ buffer: rampBuffer(0, 10), timestamp: 0 }];
    expect(assembleChunk(early, 3, 0, createBuffer)).toBeNull();
  });
});

describe('LevelAccumulator', () => {
  it('reports peak and RMS per channel every N frames, mono counting for both', () => {
    const levels = new LevelAccumulator(2, 256);
    const block = new Float32Array(128).fill(0.5);
    block[3] = -0.9;
    expect(levels.add([block])).toBeNull();
    const report = levels.add([new Float32Array(128)]);
    expect(report?.peak).toEqual([expect.closeTo(0.9, 6), expect.closeTo(0.9, 6)]);
    expect(report?.rms[0]).toBeCloseTo(Math.sqrt((127 * 0.25 + 0.81) / 256), 6);
    expect(levels.add([], 256)?.peak).toEqual([0, 0]);
  });
});

describe('AudioChunkPool', () => {
  function setup(failing = false) {
    const decoded: number[] = [];
    const disposed: string[] = [];
    const pool = new AudioChunkPool({
      maxOpenReaders: 1,
      budgetBytes: 3 * 400,
      createReader: (url): AudioChunkReader => ({
        chunk: async (index) => {
          await settle();
          if (failing) throw new Error('no sound');
          decoded.push(index);
          return { buffer: createBuffer(1, 100, 100), start: index } satisfies AudioChunk;
        },
        dispose: () => disposed.push(url),
      }),
    });
    pool.setLocator((assetId) => (assetId === 'silent' ? null : `media/${assetId}`));
    return { pool, decoded, disposed };
  }

  it('shares concurrent loads and serves chunks from the cache', async () => {
    const { pool, decoded } = setup();
    const [first, second] = await Promise.all([pool.load('a', 2), pool.load('a', 2)]);
    expect(first).toBe(second);
    expect(decoded).toEqual([2]);
    expect(pool.peek('a', 2)).toBe(first);
    expect(await pool.load('silent', 0)).toBeNull();
  });

  it('evicts the least recently used chunks beyond the budget', async () => {
    const { pool } = setup();
    for (const index of [0, 1, 2, 3]) await pool.load('a', index);
    expect(pool.stats()).toMatchObject({ chunks: 3, bytes: 1200 });
    expect(pool.peek('a', 0)).toBeNull();
  });

  it('does not retry a file without sound, and closes idle readers', async () => {
    const failing = setup(true);
    expect(await failing.pool.load('a', 0)).toBeNull();
    expect(await failing.pool.load('a', 1)).toBeNull();
    const { pool, disposed } = setup();
    await pool.load('a', 0);
    await pool.load('b', 0);
    expect(disposed).toEqual(['media/a']);
  });
});
