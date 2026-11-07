import { describe, expect, it } from 'vitest';

import { DecoderPool, type MediaFileRef } from '../../src/decode/decoder-pool';
import { FrameTracker } from '../../src/decode/frame-tracker';
import { MediabunnyVideoSource } from '../../src/decode/video-source';
import count25 from './fixtures/count-25.webm?url';
import count2997 from './fixtures/count-2997.mp4?url';

const files: MediaFileRef[] = [
  { assetId: 'ntsc', variant: 'original', url: count2997 },
  { assetId: 'pal', variant: 'original', url: count25 },
];

describe('DecoderPool with WebCodecs', () => {
  it('closes every VideoFrame after an intensive scrub with cancellations', async () => {
    const tracker = new FrameTracker();
    const pool = new DecoderPool({
      tracker,
      maxCachedPictures: 16,
      createVideoSource: (url) => new MediabunnyVideoSource(url, 'prefer-software'),
    });
    let seed = 11;
    const random = () => {
      seed = (seed * 1_103_515_245 + 12_345) % 2_147_483_648;
      return seed / 2_147_483_648;
    };
    // Like a fast scrub: bursts of requests where each new one makes the previous obsolete.
    const deadline = performance.now() + 15_000;
    let requests = 0;
    while (performance.now() < deadline && requests < 1_500) {
      const controller = new AbortController();
      const burst = Array.from({ length: 4 }, () => {
        const file = files[random() < 0.5 ? 0 : 1] ?? files[0];
        if (file === undefined) throw new Error('no fixture');
        return pool.videoFrame(file, random() * 1.9, controller.signal);
      });
      controller.abort();
      requests += burst.length;
      for (const lease of await Promise.all(burst)) lease?.release();
      expect(pool.stats().cachedPictures).toBeLessThanOrEqual(16);
    }
    expect(requests).toBeGreaterThan(100);
    expect(tracker.counts().videoFrames).toBe(pool.stats().cachedPictures);
    pool.destroy();
    expect(tracker.counts()).toEqual({ videoFrames: 0, images: 0 });
  });
});
