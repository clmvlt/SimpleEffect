import { describe, expect, it } from 'vitest';

import { draggedCaptionPosition } from './caption-drag';

describe('caption drags in the viewer', () => {
  it('move the bottom centre with the pointer, snapping to the centre and thirds', () => {
    const frame = { width: 1920, height: 1080 };
    const moved = draggedCaptionPosition(
      { x: 0.5, y: 0.9 },
      { x: 960, y: 970 },
      { x: 600, y: 700 },
      frame,
      0,
    );
    expect(moved.position.x).toBeCloseTo(0.3125);
    expect(moved.position.y).toBeCloseTo(0.65);
    const snapped = draggedCaptionPosition(
      { x: 0.5, y: 0.9 },
      { x: 960, y: 970 },
      { x: 965, y: 715 },
      frame,
      10,
    );
    expect(snapped.position).toEqual({ x: 0.5, y: 2 / 3 });
    expect(snapped.guides).toEqual({ x: [960], y: [720] });
    const clamped = draggedCaptionPosition(
      { x: 0.5, y: 0.9 },
      { x: 0, y: 0 },
      { x: 0, y: 900 },
      frame,
      0,
    );
    expect(clamped.position.y).toBe(1);
  });
});
