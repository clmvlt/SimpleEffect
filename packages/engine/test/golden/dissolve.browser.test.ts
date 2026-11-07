import type { Flicks } from '@se/protocol';
import { afterEach, describe, expect, it } from 'vitest';

import type { RenderScene } from '../../src/preview/frame-renderer';
import {
  NTSC_30,
  assetTable,
  clip,
  crossFade,
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
import count2997 from './fixtures/count-2997.mp4?url';
import { comparePixels, expectGolden, renderScene, type Rendered } from './golden';

const video = videoAsset({ width: 320, height: 180 });
const pattern = imageAsset(320, 180);
const assets = assetTable(video, pattern);
const files = { [video.id]: { original: count2997 }, [pattern.id]: { original: bars } };
/** Design pixels of a 320 × 180 frame: 1080 design pixels = 180 pixels. */
const DESIGN = 6;

let rendered: Rendered[] = [];
afterEach(() => {
  for (const entry of rendered) entry.engine.destroy();
  rendered = [];
});

async function render(scene: RenderScene, time: Flicks): Promise<ImageData> {
  const result = await renderScene(scene, time, files);
  rendered.push(result);
  return result.pixels;
}

function single(kind: Parameters<typeof item>[2]): RenderScene {
  return {
    sequence: sequence(320, 180, [videoTrack([item(0 as Flicks, seconds(3), kind)])]),
    assets,
  };
}

/** `(1 − progress)·a + progress·b`, pixel by pixel. */
function blend(a: ImageData, b: ImageData, progress: number): ImageData {
  const mixed = new ImageData(a.width, a.height);
  for (let offset = 0; offset < a.data.length; offset += 1) {
    mixed.data[offset] = Math.round(
      (1 - progress) * (a.data[offset] ?? 0) + progress * (b.data[offset] ?? 0),
    );
  }
  return mixed;
}

function pixelAt(pixels: ImageData, x: number, y: number): number[] {
  const offset = (y * pixels.width + x) * 4;
  return Array.from(pixels.data.slice(offset, offset + 3));
}

describe('golden frames: cross dissolves', () => {
  it('blends the outgoing video and the incoming picture by the progress', async () => {
    // Video `[0, 1.5 s)`, bars `[1.5 s, 3 s)`, a centred 1 s dissolve: `[1 s, 2 s)`.
    const outgoing = item(0 as Flicks, seconds(1.5), clip(video.id));
    const incoming = item(seconds(1.5), seconds(1.5), image(pattern.id));
    const dissolve = crossFade(outgoing, incoming, seconds(1));
    const scene: RenderScene = {
      sequence: sequence(320, 180, [videoTrack([outgoing, incoming], false, [dissolve])]),
      assets,
    };
    // Frame 45 of the video (1.5015 s): just past the cut, the video plays past its end.
    const time = frameTime(45, NTSC_30);
    const progress = (time - seconds(1)) / seconds(1);
    const pixels = await render(scene, time);
    const alone = blend(
      await render(single(clip(video.id)), time),
      await render(single(image(pattern.id)), time),
      progress,
    );
    expect(comparePixels(pixels, alone, { channel: 3, ratio: 0 }).ratio).toBe(0);
    await expectGolden('cross-dissolve-midway', pixels);
  });

  it('never lets the lower tracks show through halfway', async () => {
    // A red background on V1; on V2 a white square dissolving into a blue one, halfway.
    const background = item(
      0 as Flicks,
      seconds(2),
      rectangle('#ff0000', 320 * DESIGN, 180 * DESIGN),
    );
    const white = item(0 as Flicks, seconds(1), rectangle('#ffffff', 90 * DESIGN, 90 * DESIGN));
    const blue = item(seconds(1), seconds(1), rectangle('#0000ff', 90 * DESIGN, 90 * DESIGN));
    const scene: RenderScene = {
      sequence: sequence(320, 180, [
        videoTrack([background]),
        videoTrack([white, blue], false, [crossFade(white, blue, seconds(1))]),
      ]),
      assets,
    };
    const pixels = await render(scene, seconds(1));
    const [red, green, blueLevel] = pixelAt(pixels, 160, 90);
    expect(Math.abs((red ?? 0) - 128)).toBeLessThanOrEqual(2);
    expect(Math.abs((green ?? 0) - 128)).toBeLessThanOrEqual(2);
    expect(blueLevel).toBeGreaterThanOrEqual(253);
    expect(pixelAt(pixels, 10, 10)).toEqual([255, 0, 0]);
  });
});
