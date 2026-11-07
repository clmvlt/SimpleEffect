import type { AssetId, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { DecoderPool } from '../decode/decoder-pool';
import { DecodeError } from '../decode/decode-error';
import { FrameTracker } from '../decode/frame-tracker';
import type { FontLoader } from '../fonts/font-registry';
import {
  assetTable,
  clip,
  item,
  seconds,
  sequence,
  text,
  videoAsset,
  videoTrack,
} from '../testing/document';
import { FakeFrame, FakeVideoSource, settle } from '../testing/fake-decoding';
import { RecordingCompositor } from '../testing/recording-compositor';
import { createEngineWith } from './engine';
import type { FrameLayout } from './frame-layout';
import { FrameRenderer, type RenderScene } from './frame-renderer';

/** 25 fps like the fake source, so lookup times land on its frames. */
const asset = videoAsset({
  width: 1920,
  height: 1080,
  proxy: true,
  frameRate: { numerator: 25, denominator: 1 },
});
const shot = item(0 as Flicks, seconds(10), clip(asset.id));
const scene: RenderScene = {
  sequence: sequence(1920, 1080, [videoTrack([shot])]),
  assets: assetTable(asset),
};
const title = item(0 as Flicks, seconds(10), text('Titre', { style: { fontFamily: 'Oswald' } }));
const titled: RenderScene = {
  ...scene,
  sequence: sequence(1920, 1080, [videoTrack([shot]), videoTrack([title])]),
};

/** Fonts that load when `finish` is called; `Oswald` is in the catalogue, nothing else is. */
function fakeFonts() {
  let finish: () => void = () => undefined;
  const loading = new Promise<void>((resolve) => {
    finish = resolve;
  });
  let ready = false;
  const fonts: FontLoader = {
    missing: (requests) => requests.map((request) => request.family).filter((f) => f !== 'Oswald'),
    ensure: () => (ready ? null : loading),
  };
  return {
    fonts,
    finish: () => {
      ready = true;
      finish();
    },
  };
}

function setup(failure: Error | null = null) {
  const tracker = new FrameTracker();
  const sources: FakeVideoSource[] = [];
  const pool = new DecoderPool({
    tracker,
    createVideoSource: (url) => {
      const source = new FakeVideoSource(url);
      source.failure = failure;
      sources.push(source);
      return source;
    },
  });
  const compositor = new RecordingCompositor();
  const locator = (assetId: AssetId, variant: string) => `media://${assetId}/${variant}`;
  return {
    tracker,
    sources,
    pool,
    compositor,
    renderer: new FrameRenderer(compositor, pool, locator),
  };
}

function drawnFrame(compositor: RecordingCompositor, index = -1): number | undefined {
  const draw = compositor.draws.at(index);
  const picture = draw?.pictures.get(shot.id);
  return (picture?.source as FakeFrame | undefined)?.index;
}

describe('FrameRenderer', () => {
  it('decodes the pictures of a frame, then draws it and returns the leases', async () => {
    const { renderer, compositor, tracker, pool } = setup();
    let waited = false;
    const outcome = await renderer.render({ scene, time: seconds(2), proxy: false }, () => {
      waited = true;
    });
    expect(outcome).toMatchObject({ kind: 'drawn', issues: [], missingFonts: [] });
    expect(waited).toBe(true);
    expect(drawnFrame(compositor)).toBe(50);
    expect(tracker.counts().videoFrames).toBe(1);
    pool.destroy();
    expect(tracker.counts().videoFrames).toBe(0);
  });

  it('waits for the fonts of its texts and reports their boxes', async () => {
    const { renderer, compositor } = setup();
    const { fonts, finish } = fakeFonts();
    renderer.setFonts(fonts);
    await renderer.render({ scene, time: seconds(2), proxy: false });
    const drawn = compositor.draws.length;
    const pending = renderer.render({ scene: titled, time: seconds(2), proxy: false });
    await Promise.resolve();
    expect(compositor.draws).toHaveLength(drawn);
    finish();
    const outcome = await pending;
    expect(compositor.draws).toHaveLength(drawn + 1);
    if (outcome.kind !== 'drawn') throw new Error('expected a drawn frame');
    expect(outcome.missingFonts).toEqual([]);
    const titleLayout = outcome.layout.layers.find((layer) => layer.itemId === title.id);
    expect(titleLayout?.box.width).toBeGreaterThan(0);
    expect(outcome.layout.layers[0]?.box).toEqual({ width: 1920, height: 1080 });
  });

  it('reports font families the catalogue lacks', async () => {
    const { renderer } = setup();
    renderer.setFonts({ missing: () => ['Comic'], ensure: () => null });
    const outcome = await renderer.render({ scene: titled, time: seconds(2), proxy: false });
    expect(outcome).toMatchObject({ kind: 'drawn', missingFonts: ['Comic'] });
  });

  it('draws a cached frame synchronously', async () => {
    const { renderer, compositor } = setup();
    await renderer.render({ scene, time: seconds(2), proxy: false });
    let waited = false;
    const pending = renderer.render({ scene, time: seconds(2.01), proxy: false }, () => {
      waited = true;
    });
    expect(compositor.draws).toHaveLength(2);
    await pending;
    expect(waited).toBe(false);
  });

  it('drops a request a newer one replaced, drawing only the newer frame', async () => {
    const { renderer, compositor, sources } = setup();
    await renderer.render({ scene, time: 0 as Flicks, proxy: false });
    const source = sources[0];
    if (source === undefined) throw new Error('no source');
    source.manual = true;
    const first = renderer.render({ scene, time: seconds(4), proxy: false });
    await settle();
    const second = renderer.render({ scene, time: seconds(6), proxy: false });
    source.pending.shift()?.resolve();
    expect(await first).toEqual({ kind: 'superseded' });
    await settle();
    source.pending.shift()?.resolve();
    await second;
    expect(compositor.draws).toHaveLength(2);
    expect(drawnFrame(compositor)).toBe(150);
  });

  it('reads the proxy when asked and keeps the original otherwise', async () => {
    const { renderer, sources } = setup();
    await renderer.render({ scene, time: 0 as Flicks, proxy: true });
    await renderer.render({ scene, time: 0 as Flicks, proxy: false });
    expect(sources.map((source) => source.url)).toEqual([
      `media://${asset.id}/proxy`,
      `media://${asset.id}/original`,
    ]);
  });

  it('reports media it could not draw, once per asset', async () => {
    const { renderer, compositor } = setup(new DecodeError('readFailed', 'unreadable'));
    const offline = { ...asset, missing: true, derived: { mezzanine: null, proxy: null } };
    const unchecked = videoAsset({ width: 640, height: 360 });
    const checking = {
      ...unchecked,
      decodability: { video: 'unknown' as const, audio: 'noStream' as const },
    };
    const broken = videoAsset({ width: 640, height: 360 });
    const tracks = [offline, checking, broken].map((media) =>
      videoTrack([item(0 as Flicks, seconds(1), clip(media.id))]),
    );
    const outcome = await renderer.render({
      scene: {
        sequence: sequence(1920, 1080, tracks),
        assets: assetTable(offline, checking, broken),
      },
      time: 0 as Flicks,
      proxy: false,
    });
    expect(outcome.kind === 'drawn' ? outcome.issues : []).toEqual([
      { assetId: offline.id, kind: 'missing' },
      { assetId: checking.id, kind: 'preparing' },
      { assetId: broken.id, kind: 'decodeFailed', failure: 'readFailed', message: 'unreadable' },
    ]);
    expect(compositor.draws.at(-1)?.pictures.size).toBe(0);
  });
});

describe('PreviewEngine', () => {
  function engineSetup() {
    const parts = setup();
    const scheduled: (() => void)[] = [];
    const engine = createEngineWith(
      parts.compositor,
      parts.pool,
      (assetId, variant) => `${assetId}/${variant}`,
      (callback) => scheduled.push(callback),
    );
    const flush = () => {
      for (let callback = scheduled.shift(); callback !== undefined; callback = scheduled.shift())
        callback();
    };
    return { ...parts, engine, flush };
  }

  it('coalesces changes into one draw and reports loading then ready', async () => {
    const { engine, compositor, flush } = engineSetup();
    const statuses: string[] = [];
    engine.subscribe((state) => statuses.push(state.status));
    engine.setScene(scene);
    engine.setTime(seconds(1));
    engine.setTime(seconds(3));
    flush();
    await engine.settled();
    expect(compositor.draws).toHaveLength(1);
    expect(drawnFrame(compositor)).toBe(75);
    expect(statuses).toEqual(['loading', 'ready']);
    expect(engine.getState()).toEqual({
      status: 'ready',
      issues: [],
      missingFonts: [],
      error: null,
    });
  });

  it('publishes the layout of each drawn frame', async () => {
    const { engine, flush } = engineSetup();
    const layouts: FrameLayout[] = [];
    engine.subscribeLayout((layout) => layouts.push(layout));
    engine.setScene(titled);
    engine.setTime(seconds(1));
    flush();
    await engine.settled();
    expect(layouts).toHaveLength(1);
    expect(engine.getLayout()?.layers.map((layer) => layer.itemId)).toEqual([shot.id, title.id]);
  });

  it('frees every frame and the compositor on destroy', async () => {
    const { engine, compositor, tracker, flush } = engineSetup();
    engine.setScene(scene);
    engine.setTime(seconds(1));
    flush();
    await engine.settled();
    expect(engine.stats().open.videoFrames).toBe(1);
    engine.destroy();
    expect(tracker.counts().videoFrames).toBe(0);
    expect(compositor.destroyed).toBe(true);
    await expect(engine.settled()).resolves.toBeUndefined();
  });
});
