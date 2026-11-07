import type { AssetId } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { FakeFrame, FakeVideoSource, settle } from '../testing/fake-decoding';
import { DecoderPool, type DecoderPoolOptions, type MediaFileRef } from './decoder-pool';
import { DecodeError } from './decode-error';
import { FrameTracker } from './frame-tracker';

const clip: MediaFileRef = { assetId: 'asset-a', variant: 'original', url: 'u1' };

function setup(options: DecoderPoolOptions = {}) {
  const tracker = new FrameTracker();
  const sources: FakeVideoSource[] = [];
  const pool = new DecoderPool({
    tracker,
    createVideoSource: (url) => {
      const source = new FakeVideoSource(url);
      sources.push(source);
      return source;
    },
    ...options,
  });
  return { pool, tracker, sources };
}

function frameIndex(lease: { picture: { source: unknown } } | null): number | undefined {
  return (lease?.picture.source as FakeFrame | undefined)?.index;
}

describe('DecoderPool: video frames', () => {
  it('decodes the frame shown at a time and serves it again from the cache', async () => {
    const { pool, sources } = setup();
    const first = await pool.videoFrame(clip, 1.5);
    expect(frameIndex(first)).toBe(37);
    first?.release();
    const again = pool.peekVideoFrame(clip, 1.5 + 0.01);
    expect(frameIndex(again)).toBe(37);
    again?.release();
    expect(sources[0]?.decodedTimes).toHaveLength(1);
  });

  it('opens one source per asset and variant, and reopens it when the URL changes', async () => {
    const { pool, sources } = setup();
    (await pool.videoFrame(clip, 0))?.release();
    (await pool.videoFrame({ ...clip, variant: 'proxy', url: 'p1' }, 0))?.release();
    (await pool.videoFrame({ ...clip, url: 'u2' }, 0))?.release();
    expect(sources.map((source) => source.url)).toEqual(['u1', 'p1', 'u2']);
    expect(sources[0]?.disposed).toBe(true);
    expect(sources[0]?.decodedFrames[0]?.closed).toBe(true);
    expect(pool.stats().openSources).toBe(2);
  });

  it('skips decodes made obsolete by an aborted request', async () => {
    const { pool, sources } = setup();
    (await pool.videoFrame(clip, 0))?.release();
    const source = sources[0];
    if (source === undefined) throw new Error('no source');
    source.manual = true;
    const stale = new AbortController();
    const running = pool.videoFrame(clip, 1, stale.signal);
    const obsolete = pool.videoFrame(clip, 2, stale.signal);
    const wanted = pool.videoFrame(clip, 3);
    await settle();
    stale.abort();
    expect(await obsolete).toBeNull();
    source.pending.shift()?.resolve();
    await settle();
    source.pending.shift()?.resolve();
    expect(frameIndex(await wanted)).toBe(75);
    (await running)?.release();
    expect(source.decodedTimes.map(Math.floor)).toEqual([0, 1, 3]);
  });

  it('rejects with the source error', async () => {
    const { pool, sources } = setup();
    (await pool.videoFrame(clip, 0))?.release();
    const source = sources[0];
    if (source === undefined) throw new Error('no source');
    source.failure = new DecodeError('codecNotDecodable', 'cannot decode');
    await expect(pool.videoFrame(clip, 5)).rejects.toBeInstanceOf(DecodeError);
  });

  it('closes idle sources beyond the limit, least recently used first', async () => {
    const { pool, sources } = setup({ maxOpenSources: 2 });
    for (const name of ['a', 'b', 'c']) {
      (await pool.videoFrame({ ...clip, assetId: name }, 0))?.release();
    }
    expect(sources.map((source) => source.disposed)).toEqual([true, false, false]);
  });
});

describe('DecoderPool: images', () => {
  it('loads an image once for concurrent requests', async () => {
    let loads = 0;
    const { pool, tracker } = setup({
      loadImage: async () => {
        loads += 1;
        await settle();
        return new FakeFrame(0);
      },
    });
    const still: MediaFileRef = { assetId: 'logo', variant: 'original', url: 'i1' };
    const [first, second] = await Promise.all([pool.image(still), pool.image(still)]);
    expect(loads).toBe(1);
    expect(first?.picture).toBe(second?.picture);
    first?.release();
    second?.release();
    expect(tracker.counts().images).toBe(1);
    pool.destroy();
    expect(tracker.counts().images).toBe(0);
  });
});

describe('DecoderPool: frame lifetimes', () => {
  it('closes every frame after an intensive scrub with cancellations', async () => {
    const { pool, tracker, sources } = setup({ maxCachedPictures: 12 });
    let seed = 7;
    const random = () => {
      seed = (seed * 1_103_515_245 + 12_345) % 2_147_483_648;
      return seed / 2_147_483_648;
    };
    let controller = new AbortController();
    const held = [];
    for (let step = 0; step < 3_000; step += 1) {
      if (step % 3 === 0) {
        controller.abort();
        controller = new AbortController();
      }
      const assetId = (random() < 0.5 ? 'a' : 'b') as AssetId;
      const request = pool.videoFrame({ ...clip, assetId }, random() * 10, controller.signal);
      held.push(request);
      if (held.length > 8) (await held.shift())?.release();
      expect(pool.stats().cachedPictures).toBeLessThanOrEqual(12 + 8);
    }
    for (const request of held) (await request)?.release();
    expect(pool.stats().cachedPictures).toBeLessThanOrEqual(12);
    expect(tracker.counts().videoFrames).toBe(pool.stats().cachedPictures);
    pool.destroy();
    expect(tracker.counts()).toEqual({ videoFrames: 0, images: 0 });
    const decoded = sources.flatMap((source) => source.decodedFrames);
    expect(decoded.every((frame) => frame.closed)).toBe(true);
  });

  it('closes a leased frame on release after destroy', async () => {
    const { pool, tracker } = setup();
    const lease = await pool.videoFrame(clip, 1);
    pool.destroy();
    expect(tracker.counts().videoFrames).toBe(1);
    lease?.release();
    expect(tracker.counts().videoFrames).toBe(0);
    expect(await pool.videoFrame(clip, 1)).toBeNull();
  });
});
