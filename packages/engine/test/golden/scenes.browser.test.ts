import type { Flicks } from '@se/protocol';
import { afterEach, describe, expect, it } from 'vitest';

import type { RenderScene } from '../../src/preview/frame-renderer';
import {
  NTSC_30,
  assetTable,
  clip,
  ellipse,
  frameTime,
  image,
  imageAsset,
  item,
  rectangle,
  seconds,
  sequence,
  videoAsset,
  videoTrack,
} from '../../src/testing/document';
import bars from './fixtures/bars.png?url';
import count2997Proxy from './fixtures/count-2997-proxy.mp4?url';
import count2997 from './fixtures/count-2997.mp4?url';
import disc from './fixtures/disc.png?url';
import {
  comparePixels,
  createTestEngine,
  frameAt,
  expectGolden,
  luminanceAt,
  readFrameNumber,
  renderScene,
  type Rendered,
} from './golden';

const video = videoAsset({ width: 320, height: 180, proxy: true });
const pattern = imageAsset(320, 180);
const sticker = imageAsset(64, 64);
const assets = assetTable(video, pattern, sticker);
const files = {
  [video.id]: { original: count2997, proxy: count2997Proxy },
  [pattern.id]: { original: bars },
  [sticker.id]: { original: disc },
};
const ALL = seconds(5);
const at = (kind: Parameters<typeof item>[2]) => item(0 as Flicks, ALL, kind);
const scene = (
  width: number,
  height: number,
  ...layers: Parameters<typeof item>[2][]
): RenderScene => ({
  sequence: sequence(
    width,
    height,
    layers.map((layer) => videoTrack([at(layer)])),
  ),
  assets,
});

let rendered: Rendered[] = [];
afterEach(() => {
  rendered.forEach((entry) => {
    entry.engine.destroy();
  });
  rendered = [];
});

async function render(
  target: RenderScene,
  time: Flicks,
  options: Parameters<typeof renderScene>[3] = {},
) {
  const result = await renderScene(target, time, files, options);
  rendered.push(result);
  return result.pixels;
}

/** Design pixels of a 320 × 180 frame: 1080 design pixels = 180 pixels. */
const DESIGN = 6;
const shapes = scene(
  320,
  180,
  rectangle('#c0392b', 320 * DESIGN, 180 * DESIGN),
  rectangle('#2e86de', 160 * DESIGN, 90 * DESIGN, {
    cornerRadius: 12 * DESIGN,
    transform: { opacity: 0.5 },
  }),
  ellipse('#27ae60', 50 * DESIGN, 50 * DESIGN, { position: { x: 0.8, y: 0.3 } }),
);

describe('golden frames', () => {
  it('draws flat shapes with opacity, rounded corners and ellipses', async () => {
    await expectGolden('shapes-opacity', await render(shapes, 0 as Flicks));
  });

  it('draws the same frame on an OffscreenCanvas', async () => {
    await expectGolden('shapes-opacity', await render(shapes, 0 as Flicks, { offscreen: true }));
  });

  it('draws a test pattern image fitted to the frame', async () => {
    await expectGolden(
      'test-pattern',
      await render(scene(320, 180, image(pattern.id)), 0 as Flicks),
    );
  });

  it('draws a decoded video frame', async () => {
    const pixels = await render(scene(320, 180, clip(video.id)), frameTime(45, NTSC_30));
    expect(readFrameNumber(pixels, { x: 0, y: 0, width: 320, height: 180 })).toBe(45);
    await expectGolden('video-frame-45', pixels);
  });

  it('stacks tracks with transforms, mirroring and opacity', async () => {
    const layered = scene(
      320,
      180,
      clip(video.id),
      image(pattern.id, {
        position: { x: 0.3, y: 0.35 },
        scale: { x: 0.4, y: 0.4 },
        rotation: 15,
        opacity: 0.8,
      }),
      image(pattern.id, { position: { x: 0.75, y: 0.7 }, scale: { x: -0.3, y: 0.3 } }),
      image(sticker.id, {
        position: { x: 0.05, y: 0.95 },
        anchor: { x: 0, y: 1 },
        scale: { x: 0.25, y: 0.25 },
      }),
    );
    await expectGolden('multitrack-transforms', await render(layered, frameTime(30, NTSC_30)));
  });

  it('letterboxes a 16:9 clip in a 9:16 frame (fit) and crops it (fill)', async () => {
    const time = frameTime(10, NTSC_30);
    await expectGolden('reframe-fit-vertical', await render(scene(180, 320, clip(video.id)), time));
    const filled = scene(180, 320, clip(video.id, { reframe: 'fill' }));
    await expectGolden('reframe-fill-vertical', await render(filled, time));
  });

  it('draws keyframed properties at their interpolated values', async () => {
    const eased = { kind: 'preset', name: 'easeInOut' } as const;
    const animated = scene(
      320,
      180,
      rectangle('#f1c40f', 60 * DESIGN, 60 * DESIGN, {
        transform: {
          position: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: { x: 0.1, y: 0.5 }, easing: eased },
              { time: seconds(2), value: { x: 0.9, y: 0.5 }, easing: eased },
            ],
          },
          rotation: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: 0, easing: { kind: 'linear' } },
              { time: seconds(2), value: 90, easing: { kind: 'linear' } },
            ],
          },
          opacity: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: 1, easing: { kind: 'hold' } },
              { time: seconds(1), value: 0.4, easing: { kind: 'linear' } },
            ],
          },
        },
      }),
    );
    await expectGolden('keyframes-midway', await render(animated, seconds(0.5)));
  });

  it('composites an image with alpha over a video frame', async () => {
    const over = scene(320, 180, clip(video.id), image(sticker.id, { scale: { x: 0.5, y: 0.5 } }));
    await expectGolden('alpha-over-video', await render(over, frameTime(60, NTSC_30)));
  });

  it('keeps the framing when the proxy is toggled on the same engine', async () => {
    const framed = scene(
      320,
      180,
      clip(video.id, { transform: { scale: { x: 0.6, y: 0.6 }, rotation: -8 } }),
    );
    const time = frameTime(45, NTSC_30);
    const { engine, canvas } = await createTestEngine(framed, files);
    rendered.push({ engine, canvas, pixels: new ImageData(1, 1) });
    const original = await frameAt(engine, canvas, time);
    engine.setProxy(true);
    const proxy = await frameAt(engine, canvas, time);
    // Same box, same place: only resampling differs (a half-resolution proxy blurs edges).
    expect(comparePixels(proxy, original, { channel: 96, ratio: 0 }).ratio).toBeLessThan(0.03);
    await expectGolden('proxy-framing', proxy);
    engine.setProxy(false);
    const back = await frameAt(engine, canvas, time);
    expect(comparePixels(back, original, { channel: 2, ratio: 0 }).ratio).toBe(0);
  });

  it('leaves the frame transparent behind the layers when asked', async () => {
    const pixels = await render(scene(320, 180, image(sticker.id)), 0 as Flicks, {
      transparent: true,
    });
    expect(pixels.data[3]).toBe(0);
    expect(pixels.data[(90 * 320 + 160) * 4 + 3]).toBe(255);
  });

  it('draws basic text centred on its anchor', async () => {
    const titled = scene(320, 180, {
      kind: 'text',
      content: 'SIMPLE',
      style: { fontFamily: 'Inter', fontSize: 40 * DESIGN, color: '#ffffff' },
      transform: rectangleTransform(),
    });
    const pixels = await render(titled, 0 as Flicks);
    // Font rasterisation differs per platform: check placement, not glyph pixels.
    expect(luminanceAt(pixels, 5, 5)).toBe(0);
    let lit = 0;
    for (let x = 100; x < 220; x += 2) if (luminanceAt(pixels, x, 90) > 128) lit += 1;
    expect(lit).toBeGreaterThan(5);
    expect(luminanceAt(pixels, 20, 90)).toBe(0);
  });
});

function rectangleTransform() {
  const shape = rectangle('#000000', 1, 1);
  if (shape.kind !== 'shape') throw new Error('unreachable');
  return shape.transform;
}
