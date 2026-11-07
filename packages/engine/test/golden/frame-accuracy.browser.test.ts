import type { AssetId, Flicks } from '@se/protocol';
import { afterEach, describe, expect, it } from 'vitest';

import type { PreviewEngine } from '../../src/preview/engine';
import {
  NTSC_30,
  assetTable,
  clip,
  frameTime,
  item,
  seconds,
  sequence,
  videoAsset,
  videoTrack,
} from '../../src/testing/document';
import count25 from './fixtures/count-25.webm?url';
import count2997Proxy from './fixtures/count-2997-proxy.mp4?url';
import count2997Rotated from './fixtures/count-2997-rotated.mp4?url';
import count2997 from './fixtures/count-2997.mp4?url';
import { createTestEngine, frameAt, readFrameNumber } from './golden';

const FULL_FRAME = { x: 0, y: 0, width: 320, height: 180 };
const ntsc = videoAsset({ width: 320, height: 180, proxy: true, duration: seconds(3) });
const pal = videoAsset({ width: 320, height: 180, frameRate: { numerator: 25, denominator: 1 } });
const phone = videoAsset({ width: 320, height: 180, rotation: 90 });
const files = {
  [ntsc.id]: { original: count2997, proxy: count2997Proxy },
  [pal.id]: { original: count25 },
  [phone.id]: { original: count2997Rotated },
};

let engines: PreviewEngine[] = [];
afterEach(() => {
  engines.forEach((engine) => {
    engine.destroy();
  });
  engines = [];
});

async function engineFor(
  assetId: AssetId,
  options: { sourceStart?: Flicks; speed?: number } = {},
  proxy = false,
) {
  const shot = item(0 as Flicks, seconds(10), clip(assetId, options));
  const scene = {
    sequence: sequence(320, 180, [videoTrack([shot])]),
    assets: assetTable(ntsc, pal, phone),
  };
  const created = await createTestEngine(scene, files, { proxy });
  engines.push(created.engine);
  return created;
}

describe('frame-accurate scrubbing', () => {
  it('shows exactly frame n at the time of frame n of a 29.97 fps clip', async () => {
    const { engine, canvas } = await engineFor(ntsc.id);
    for (const index of [0, 1, 2, 3, 29, 30, 31, 44, 45, 59, 60, 61, 88, 89, 12, 7]) {
      const pixels = await frameAt(engine, canvas, frameTime(index, NTSC_30));
      expect(readFrameNumber(pixels, FULL_FRAME), `frame ${String(index)}`).toBe(index);
    }
  });

  it('holds a frame until the exact start of the next one', async () => {
    const { engine, canvas } = await engineFor(ntsc.id);
    const justBefore = (frameTime(31, NTSC_30) - 1) as Flicks;
    expect(readFrameNumber(await frameAt(engine, canvas, justBefore), FULL_FRAME)).toBe(30);
    expect(readFrameNumber(await frameAt(engine, canvas, frameTime(31, NTSC_30)), FULL_FRAME)).toBe(
      31,
    );
  });

  it('maps another sequence rate, a source offset and a speed to the right source frame', async () => {
    const offset = await engineFor(ntsc.id, { sourceStart: frameTime(10, NTSC_30) });
    expect(
      readFrameNumber(await frameAt(offset.engine, offset.canvas, seconds(1)), FULL_FRAME),
    ).toBe(39);
    const fast = await engineFor(ntsc.id, { speed: 2 });
    expect(
      readFrameNumber(await frameAt(fast.engine, fast.canvas, frameTime(20, NTSC_30)), FULL_FRAME),
    ).toBe(40);
    // 0.52 s of a 25 fps sequence = 15.58 frames at 29.97: frame 15 is on screen.
    expect(
      readFrameNumber(
        await frameAt(offset.engine, offset.canvas, seconds(0.52 - 1 / 3)),
        FULL_FRAME,
      ),
    ).toBe(15);
  });

  it('decodes VP9 at 25 fps frame-accurately', async () => {
    const { engine, canvas } = await engineFor(pal.id);
    for (const index of [0, 1, 24, 25, 26, 49, 13]) {
      const pixels = await frameAt(engine, canvas, seconds(index / 25));
      expect(readFrameNumber(pixels, FULL_FRAME), `frame ${String(index)}`).toBe(index);
    }
  });

  it('shows the same frame from the proxy', async () => {
    const { engine, canvas } = await engineFor(ntsc.id, {}, true);
    for (const index of [0, 17, 45, 89]) {
      const pixels = await frameAt(engine, canvas, frameTime(index, NTSC_30));
      expect(readFrameNumber(pixels, FULL_FRAME), `proxy frame ${String(index)}`).toBe(index);
    }
  });

  it('turns phone footage upright', async () => {
    const shot = item(0 as Flicks, seconds(3), clip(phone.id));
    const scene = { sequence: sequence(180, 320, [videoTrack([shot])]), assets: assetTable(phone) };
    const { engine, canvas } = await createTestEngine(scene, files);
    engines.push(engine);
    const pixels = await frameAt(engine, canvas, frameTime(45, NTSC_30));
    expect(readFrameNumber(pixels, { x: 0, y: 0, width: 180, height: 320 }, true)).toBe(45);
  });
});
