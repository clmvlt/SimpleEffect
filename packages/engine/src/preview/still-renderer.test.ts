import type { AssetId, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { DecoderPool } from '../decode/decoder-pool';
import { FrameTracker } from '../decode/frame-tracker';
import {
  assetTable,
  clip,
  item,
  seconds,
  sequence,
  videoAsset,
  videoTrack,
} from '../testing/document';
import { FakeVideoSource, type FakeFrame } from '../testing/fake-decoding';
import { RecordingCompositor } from '../testing/recording-compositor';
import type { RenderScene } from './frame-renderer';
import { STILL_JPEG_QUALITY, createStillRendererWith } from './still-renderer';

const asset = videoAsset({
  width: 1920,
  height: 1080,
  frameRate: { numerator: 25, denominator: 1 },
});
const shot = item(0 as Flicks, seconds(10), clip(asset.id));
const scene: RenderScene = {
  sequence: sequence(1920, 1080, [videoTrack([shot])]),
  assets: assetTable(asset),
};

/** A canvas whose encoder records the options it was asked for. */
function fakeCanvas() {
  const encodings: ImageEncodeOptions[] = [];
  const canvas = {
    convertToBlob: (options: ImageEncodeOptions) => {
      encodings.push(options);
      return Promise.resolve(
        new Blob([`still ${String(encodings.length)}`], { type: 'image/jpeg' }),
      );
    },
  } as unknown as OffscreenCanvas;
  return { canvas, encodings };
}

function setup() {
  const tracker = new FrameTracker();
  const pool = new DecoderPool({ tracker, createVideoSource: (url) => new FakeVideoSource(url) });
  const compositor = new RecordingCompositor();
  const { canvas, encodings } = fakeCanvas();
  const locator = (assetId: AssetId, variant: string) => `media://${assetId}/${variant}`;
  const stills = createStillRendererWith(canvas, compositor, pool, { locator });
  return { tracker, compositor, encodings, stills };
}

describe('StillRenderer', () => {
  it('draws every still asked at once, in order, as JPEG', async () => {
    const { stills, compositor, encodings } = setup();
    const [first, second] = await Promise.all([
      stills.render(scene, seconds(2), false),
      stills.render(scene, seconds(4), false),
    ]);
    expect(await first.jpeg.text()).toBe('still 1');
    expect(await second.jpeg.text()).toBe('still 2');
    expect(first.issues).toEqual([]);
    const frames = compositor.draws.map(
      (draw) => (draw.pictures.get(shot.id)?.source as FakeFrame | undefined)?.index,
    );
    expect(frames).toEqual([50, 100]);
    expect(encodings).toEqual([
      { type: 'image/jpeg', quality: STILL_JPEG_QUALITY },
      { type: 'image/jpeg', quality: STILL_JPEG_QUALITY },
    ]);
  });

  it('frees every picture and the compositor when destroyed', async () => {
    const { stills, compositor, tracker } = setup();
    await stills.render(scene, seconds(2), false);
    expect(tracker.counts().videoFrames).toBe(1);
    stills.destroy();
    expect(tracker.counts().videoFrames).toBe(0);
    expect(compositor.destroyed).toBe(true);
  });
});
