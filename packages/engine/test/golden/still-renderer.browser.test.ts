import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { createStillRenderer } from '../../src/preview/still-renderer';
import type { RenderScene } from '../../src/preview/frame-renderer';
import { item, rectangle, seconds, sequence, videoTrack } from '../../src/testing/document';

/** A 1280 × 720 sequence: a red frame-covering plate with a centred blue square. */
const scene: RenderScene = {
  sequence: sequence(1280, 720, [
    videoTrack([item(0 as Flicks, seconds(2), rectangle('#d00000', 10, 10, { coverFrame: true }))]),
    videoTrack([item(0 as Flicks, seconds(2), rectangle('#0000d0', 360, 360))]),
  ]),
  assets: {},
};

/** The colour of the pixel at `(x, y)` of a decoded JPEG. */
async function pixelAt(jpeg: Blob, x: number, y: number): Promise<readonly number[]> {
  const bitmap = await createImageBitmap(jpeg);
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  const context = canvas.getContext('2d');
  if (context === null) throw new Error('no 2D context');
  context.drawImage(bitmap, 0, 0);
  bitmap.close();
  return [...context.getImageData(x, y, 1, 1).data.slice(0, 3)];
}

describe('StillRenderer', () => {
  it('encodes the composited frame as a JPEG of the asked size', async () => {
    const stills = await createStillRenderer({
      width: 384,
      height: 216,
      locator: () => null,
    });
    try {
      const still = await stills.render(scene, seconds(1), false);
      expect(still.jpeg.type).toBe('image/jpeg');
      const bitmap = await createImageBitmap(still.jpeg);
      expect([bitmap.width, bitmap.height]).toEqual([384, 216]);
      bitmap.close();
      const [red, , blueOnRed] = await pixelAt(still.jpeg, 10, 10);
      expect(red).toBeGreaterThan(180);
      expect(blueOnRed).toBeLessThan(60);
      const [centreRed, , centreBlue] = await pixelAt(still.jpeg, 192, 108);
      expect(centreBlue).toBeGreaterThan(180);
      expect(centreRed).toBeLessThan(60);
    } finally {
      stills.destroy();
    }
  });
});
