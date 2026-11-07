import type { Flicks, ItemKind } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { clipKind, fill, videoAsset } from '../../../shared/project-store/testing/reframe-fixture';
import { reframeEdit } from './reframe-switch';

const hd = { width: 1920, height: 1080 };
const uhd = videoAsset('asset-video', 3840, 2160);

function withScale(kind: ItemKind, x: number): ItemKind {
  if (kind.kind !== 'mediaClip') return kind;
  return {
    ...kind,
    transform: {
      ...kind.transform,
      scale: {
        kind: 'keyframed',
        keyframes: [
          { time: 0 as Flicks, value: { x, y: x }, easing: { kind: 'linear' } },
          { time: 1000 as Flicks, value: { x: 2 * x, y: 2 * x }, easing: { kind: 'linear' } },
        ],
      },
    },
  };
}

describe('reframeEdit', () => {
  it('does nothing for the current mode or another kind', () => {
    expect(reframeEdit(clipKind({ kind: 'fit' }), 'fit', uhd, hd)).toBeNull();
  });

  it('keeps the focus of a tracked clip and the scale between fit and fill', () => {
    const tracked = clipKind({ ...fill({ x: 0.2, y: 0.5 }), kind: 'tracked' });
    expect(reframeEdit(tracked, 'fill', uhd, hd)).toEqual({
      patch: { kind: 'mediaClip', reframe: fill({ x: 0.2, y: 0.5 }) },
      animations: [],
    });
    expect(reframeEdit(clipKind({ kind: 'fit' }), 'fill', uhd, hd)?.animations).toEqual([]);
  });

  it('scales a 4K clip by one half on its way to its natural size, every keyframe included', () => {
    const edit = reframeEdit(withScale(clipKind({ kind: 'fit' }), 1), 'custom', uhd, hd);
    expect(edit?.patch).toEqual({ kind: 'mediaClip', reframe: { kind: 'custom' } });
    expect(edit?.animations).toEqual([
      {
        property: 'scale',
        value: {
          kind: 'keyframed',
          keyframes: [
            { time: 0, value: { x: 0.5, y: 0.5 }, easing: { kind: 'linear' } },
            { time: 1000, value: { x: 1, y: 1 }, easing: { kind: 'linear' } },
          ],
        },
      },
    ]);
  });

  it('scales back up when leaving the natural size', () => {
    const custom = clipKind({ kind: 'custom' });
    const edit = reframeEdit(custom, 'fill', uhd, { width: 1080, height: 1920 });
    // Natural: 3840 design px = 3840 px; fill of a 9:16 frame: 1920 / 2160 px per pixel.
    expect(edit?.animations[0]).toMatchObject({
      property: 'scale',
      value: { kind: 'static', value: { x: 1 / (1920 / 2160), y: 1 / (1920 / 2160) } },
    });
  });
});
