import { describe, expect, it } from 'vitest';

import { FakeFrame, FakeVideoSource, settle } from '../testing/fake-decoding';
import { DecoderPool, type MediaFileRef, type PrefetchRequest } from './decoder-pool';
import { FrameTracker } from './frame-tracker';
import { MAX_READER_LAG_SECONDS, planRead } from './read-plan';

const clip: MediaFileRef = { assetId: 'asset-a', variant: 'original', url: 'u1' };

function setup(maxCachedPictures = 48) {
  const tracker = new FrameTracker();
  const sources: FakeVideoSource[] = [];
  const pool = new DecoderPool({
    tracker,
    maxCachedPictures,
    createVideoSource: (url) => {
      const source = new FakeVideoSource(url);
      sources.push(source);
      return source;
    },
  });
  return { pool, tracker, sources };
}

/** Forward window of `seconds` from `from` (25 fps fake source: 0.04 s per frame). */
function forward(from: number, seconds = 0.4, key = 'item-1'): PrefetchRequest {
  return { key, file: clip, from, to: from + seconds, direction: 1 };
}

async function drain(): Promise<void> {
  for (let round = 0; round < 5; round += 1) await settle();
}

function indexOf(lease: { picture: { source: unknown } } | null): number | undefined {
  return (lease?.picture.source as FakeFrame | undefined)?.index;
}

describe('planRead', () => {
  it('keeps a forward reader that is close, restarts one far behind or after a jump back', () => {
    const window = { from: 2, to: 2.5, direction: 1 as const };
    expect(planRead({ start: 1, position: 1.9 }, window)).toEqual({ kind: 'keep' });
    expect(planRead({ start: 1, position: 2 - MAX_READER_LAG_SECONDS - 0.1 }, window)).toEqual({
      kind: 'restart',
      from: 2,
      to: 2.5,
    });
    expect(planRead({ start: 3, position: 3.2 }, window).kind).toBe('restart');
    expect(planRead(null, window).kind).toBe('restart');
  });

  it('reads reverse windows in chunks ending where the previous chunk started', () => {
    const window = { from: 4, to: 5, direction: -1 as const };
    expect(planRead({ start: 4.5, position: 5 }, window)).toEqual({ kind: 'keep' });
    expect(planRead({ start: 4.8, position: 5.2 }, window)).toEqual({
      kind: 'restart',
      from: 4,
      to: 4.8,
    });
    expect(planRead(null, window)).toEqual({ kind: 'restart', from: 4, to: 5 });
  });
});

describe('DecoderPool: playback readers', () => {
  it('decodes ahead with one sequential read, then serves frames from the cache', async () => {
    const { pool, sources } = setup();
    pool.prefetch([forward(1)]);
    await drain();
    const source = sources[0];
    expect(source?.readStarts).toEqual([1]);
    expect(source?.readIndexes).toEqual([25, 26, 27, 28, 29, 30, 31, 32, 33, 34]);
    expect(source?.decodedTimes).toEqual([]);
    const lease = await pool.playbackFrame(clip, 'item-1', 1.1);
    expect(indexOf(lease)).toBe(27);
    lease?.release();
    expect(pool.stats().readers).toBe(1);
  });

  it('extends the read as the window moves instead of restarting it', async () => {
    const { pool, sources } = setup();
    pool.prefetch([forward(1)]);
    await drain();
    pool.prefetch([forward(1.2)]);
    await drain();
    expect(sources[0]?.readStarts).toEqual([1]);
    expect(sources[0]?.readIndexes.at(-1)).toBe(39);
  });

  it('never holds more frames ahead than its share of the cache', async () => {
    const { pool, sources } = setup(8);
    pool.prefetch([forward(0, 10)]);
    await drain();
    expect(sources[0]?.readIndexes).toHaveLength(6);
    expect(pool.stats().cachedPictures).toBe(6);
  });

  it('waits for the reader rather than decoding the same frame twice', async () => {
    const { pool, sources } = setup();
    const source = new FakeVideoSource('u1');
    source.manual = true;
    const manualPool = new DecoderPool({
      tracker: new FrameTracker(),
      createVideoSource: () => source,
    });
    manualPool.prefetch([forward(2)]);
    const waiting = manualPool.playbackFrame(clip, 'item-1', 2.01);
    await settle();
    source.pending.shift()?.resolve();
    expect(indexOf(await waiting)).toBe(50);
    expect(source.decodedTimes).toEqual([]);
    manualPool.destroy();
    pool.destroy();
    expect(sources).toHaveLength(0);
  });

  it('falls back to random access for a time no reader covers', async () => {
    const { pool, sources } = setup();
    const lease = await pool.playbackFrame(clip, 'item-1', 3);
    expect(indexOf(lease)).toBe(75);
    lease?.release();
    expect(sources[0]?.decodedTimes).toEqual([3]);
  });

  it('stops readers no longer requested and leaks nothing on destroy', async () => {
    const { pool, tracker } = setup();
    pool.prefetch([forward(0), forward(0, 0.4, 'item-2')]);
    await drain();
    expect(pool.stats().readers).toBe(2);
    pool.prefetch([]);
    expect(pool.stats().readers).toBe(0);
    pool.prefetch([forward(5)]);
    await drain();
    pool.destroy();
    await drain();
    expect(tracker.counts()).toEqual({ videoFrames: 0, images: 0 });
  });
});
