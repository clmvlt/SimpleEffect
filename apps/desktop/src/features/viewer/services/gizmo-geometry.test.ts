import { describe, expect, it } from 'vitest';

import {
  cornersOf,
  hitTest,
  moved,
  reanchored,
  rotated,
  scaled,
  snapToFrame,
  toFrame,
  toLocal,
  type Placement,
} from './gizmo-geometry';

const frame = { width: 1920, height: 1080 };
const box = { width: 400, height: 200 };
const centred: Placement = {
  position: { x: 0.5, y: 0.5 },
  scale: { x: 1, y: 1 },
  rotation: 0,
  anchor: { x: 0.5, y: 0.5 },
};

function close(actual: { x: number; y: number }, expected: { x: number; y: number }) {
  expect(actual.x).toBeCloseTo(expected.x, 6);
  expect(actual.y).toBeCloseTo(expected.y, 6);
}

describe('gizmo geometry', () => {
  it('maps box points to the frame and back, with rotation and scale', () => {
    const turned = { ...centred, rotation: 90, scale: { x: 2, y: 1 } };
    close(toFrame(turned, box, frame, { x: 400, y: 100 }), { x: 960, y: 940 });
    close(toLocal(turned, box, frame, { x: 960, y: 940 }) ?? { x: NaN, y: NaN }, {
      x: 400,
      y: 100,
    });
    const corners = cornersOf({ itemId: 'a', transform: centred, box }, frame);
    close(corners[0] ?? { x: 0, y: 0 }, { x: 760, y: 440 });
    close(corners[2] ?? { x: 0, y: 0 }, { x: 1160, y: 640 });
  });

  it('hits the topmost layer under the pointer', () => {
    const layers = [
      { itemId: 'below', transform: centred, box: { width: 1000, height: 600 } },
      { itemId: 'above', transform: centred, box },
    ];
    expect(hitTest(layers, frame, { x: 960, y: 540 })).toBe('above');
    expect(hitTest(layers, frame, { x: 600, y: 300 })).toBe('below');
    expect(hitTest(layers, frame, { x: 10, y: 10 })).toBeNull();
  });

  it('moves, scales around the anchor and rotates around it', () => {
    close(moved(centred, frame, { x: 192, y: -108 }).position, { x: 0.6, y: 0.4 });
    // The right edge (200 px from the anchor) dragged 100 px further: 1.5×.
    expect(scaled(centred, box, frame, 'e', { x: 1260, y: 540 }, false).scale).toEqual({
      x: 1.5,
      y: 1,
    });
    // A corner with Shift keeps the ratio, taking the larger change.
    const corner = scaled(centred, box, frame, 'se', { x: 1360, y: 700 }, true).scale;
    expect(corner.x).toBeCloseTo(2);
    expect(corner.y).toBeCloseTo(2);
    const turned = rotated(centred, frame, { x: 1060, y: 540 }, { x: 960, y: 640 }, false);
    expect(turned.rotation).toBeCloseTo(90);
    const snapped = rotated(centred, frame, { x: 1060, y: 540 }, { x: 1060, y: 560 }, true);
    expect(snapped.rotation).toBe(15);
  });

  it('moves the anchor without moving the layer', () => {
    const anchored = reanchored(centred, box, frame, { x: 760, y: 440 });
    close(anchored.anchor, { x: 0, y: 0 });
    close(anchored.position, { x: 760 / 1920, y: 440 / 1080 });
    const before = cornersOf({ itemId: 'a', transform: centred, box }, frame);
    const after = cornersOf({ itemId: 'a', transform: anchored, box }, frame);
    before.forEach((corner, index) => {
      close(after[index] ?? { x: 0, y: 0 }, corner);
    });
  });

  it('snaps edges and centres to the frame lines within the threshold', () => {
    const nearCentre = moved(centred, frame, { x: 6, y: 30 });
    const { placement, guides } = snapToFrame(nearCentre, box, frame, 8);
    close(placement.position, { x: 0.5, y: 0.5 + 30 / 1080 });
    expect(guides).toEqual({ x: [960], y: [] });
    const nearLeft = { ...centred, position: { x: 205 / 1920, y: 0.5 } };
    expect(snapToFrame(nearLeft, box, frame, 8).guides.x).toEqual([0]);
  });
});
