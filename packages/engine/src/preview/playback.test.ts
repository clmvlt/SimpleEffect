import type { AssetId, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { DecoderPool } from '../decode/decoder-pool';
import { FrameTracker } from '../decode/frame-tracker';
import { frameStart } from '../playback/frame-pacing';
import {
  assetTable,
  clip,
  item,
  seconds,
  sequence,
  videoAsset,
  videoTrack,
} from '../testing/document';
import { FakeFrame, FakeVideoSource, settle } from '../testing/fake-decoding';
import { RecordingCompositor } from '../testing/recording-compositor';
import { createEngineWith } from './engine';
import type { RenderScene } from './frame-renderer';
import { prefetchRequests, prefetchSpans } from './prefetch-plan';
import { PrefetchWindow } from './prefetch-window';

const RATE = { numerator: 25, denominator: 1 };
const asset = videoAsset({ width: 1920, height: 1080, frameRate: RATE });
const other = videoAsset({ width: 1920, height: 1080, frameRate: RATE });
const first = item(0 as Flicks, seconds(4), clip(asset.id, { sourceStart: seconds(1) }));
const second = item(seconds(4), seconds(4), clip(other.id, { speed: 2 }));
const scene: RenderScene = {
  sequence: sequence(1920, 1080, [videoTrack([first, second])]),
  assets: assetTable(asset, other),
};
const locator = (assetId: AssetId, variant: string) => `media://${assetId}/${variant}`;

async function drain(): Promise<void> {
  for (let round = 0; round < 6; round += 1) await settle();
}

function setup() {
  const tracker = new FrameTracker();
  const sources: FakeVideoSource[] = [];
  const pool = new DecoderPool({
    tracker,
    createVideoSource: (url) => {
      const source = new FakeVideoSource(url);
      sources.push(source);
      return source;
    },
  });
  const compositor = new RecordingCompositor();
  let now = 0;
  const engine = createEngineWith(
    compositor,
    pool,
    locator,
    () => undefined,
    () => now,
  );
  engine.setScene(scene);
  return {
    engine,
    pool,
    tracker,
    sources,
    compositor,
    advance: (ms: number) => (now += ms),
    drawn: () =>
      compositor.draws.map(
        (draw) => (draw.pictures.values().next().value?.source as FakeFrame).index,
      ),
  };
}

describe('prefetchSpans', () => {
  it('covers the clip on screen and the next one, at their speeds', () => {
    const spans = prefetchSpans(scene.sequence, seconds(3.5), 1, 1);
    expect(spans).toEqual([
      { itemId: first.id, assetId: asset.id, from: seconds(4.5), to: seconds(5) - 1 },
      { itemId: second.id, assetId: other.id, from: 0, to: seconds(1) - 2 },
    ]);
  });

  it('looks before the time in reverse and skips hidden tracks', () => {
    const spans = prefetchSpans(scene.sequence, seconds(5), -1, 0.5);
    expect(spans).toEqual([
      { itemId: second.id, assetId: other.id, from: seconds(1), to: seconds(2) },
    ]);
    const hidden = { ...scene.sequence, tracks: [videoTrack([first], true)] };
    expect(prefetchSpans(hidden, 0 as Flicks, 1, 1)).toEqual([]);
  });

  it('shortens reverse windows to what one reader may hold', () => {
    const [request] = prefetchRequests(
      [{ itemId: first.id, assetId: asset.id, from: seconds(1), to: seconds(3) }],
      { assets: scene.assets, proxy: false, locator, direction: -1, maxFrames: 10 },
    );
    expect(request?.to).toBeCloseTo(3.04, 6);
    expect(request?.from).toBeCloseTo(2.64, 6);
  });
});

describe('PrefetchWindow', () => {
  it('grows on late frames and shrinks back after calm playback', () => {
    const window = new PrefetchWindow();
    window.late(0);
    window.late(10);
    expect(window.seconds).toBeCloseTo(1.125, 6);
    window.onTime(1000);
    expect(window.seconds).toBeCloseTo(1.125, 6);
    window.onTime(3100);
    expect(window.seconds).toBeCloseTo(0.9, 6);
  });
});

describe('PreviewEngine playback', () => {
  it('decodes ahead sequentially and draws cached frames in the same refresh', async () => {
    const { engine, sources, compositor, drawn } = setup();
    engine.setTime(seconds(1));
    engine.setPlayback({ rate: 1, frameRate: RATE });
    await drain();
    expect(sources[0]?.readStarts).toHaveLength(1);
    const before = compositor.draws.length;
    engine.setTime(frameStart(26, RATE));
    expect(compositor.draws).toHaveLength(before + 1);
    expect(drawn().at(-1)).toBe(51);
    expect(sources[0]?.decodedTimes).toEqual([]);
  });

  it('never supersedes a late frame: it is drawn, then the latest time', async () => {
    const { engine, sources, drawn } = setup();
    engine.setPlayback({ rate: 1, frameRate: RATE });
    engine.setTime(seconds(2));
    await drain();
    const source = sources[0];
    if (source === undefined) throw new Error('no source');
    source.manual = true;
    engine.setTime(seconds(3.6));
    engine.setTime(seconds(3.64));
    engine.setTime(seconds(3.68));
    await drain();
    while (source.pending.length > 0) {
      source.pending.shift()?.resolve();
      await drain();
    }
    expect(drawn().slice(-2)).toEqual([115, 117]);
  });

  it('counts frames drawn and skipped, and stops readers on pause', async () => {
    const { engine, pool, advance } = setup();
    engine.setPlayback({ rate: 1, frameRate: RATE });
    for (const index of [1, 2, 3, 6, 7]) {
      advance(40);
      engine.setTime(frameStart(index, RATE));
      await drain();
    }
    expect(engine.stats()).toMatchObject({ droppedFrames: 2, framesPerSecond: 5 });
    expect(pool.stats().readers).toBeGreaterThan(0);
    engine.setPlayback(null);
    expect(pool.stats().readers).toBe(0);
  });

  it('leaks no frame after playing and destroying', async () => {
    const { engine, tracker } = setup();
    engine.setPlayback({ rate: 2, frameRate: RATE });
    for (let index = 0; index < 60; index += 3) {
      engine.setTime(frameStart(index, RATE));
      await settle();
    }
    engine.destroy();
    await drain();
    expect(tracker.counts()).toEqual({ videoFrames: 0, images: 0 });
  });
});
