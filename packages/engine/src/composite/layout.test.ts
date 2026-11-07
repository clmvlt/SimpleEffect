import { describe, expect, it } from 'vitest';

import type { ResolvedTransform } from '../evaluate/frame-graph';
import { codedCrop, layerPlacement, outputScale, pictureFit } from './layout';

const frame = { width: 1920, height: 1080 };

function transform(overrides: Partial<ResolvedTransform> = {}): ResolvedTransform {
  return {
    position: { x: 0.5, y: 0.5 },
    scale: { x: 1, y: 1 },
    rotation: 0,
    anchor: { x: 0.5, y: 0.5 },
    opacity: 1,
    blur: 0,
    ...overrides,
  };
}

describe('layerPlacement', () => {
  it('puts the anchor of the box at the normalised position', () => {
    const placement = layerPlacement(
      transform({ position: { x: 0.25, y: 1 }, anchor: { x: 0, y: 1 } }),
      { width: 400, height: 200 },
      frame,
    );
    expect(placement).toMatchObject({ x: 480, y: 1080, pivotX: 0, pivotY: 200 });
  });

  it('keeps scale, converts clockwise degrees to radians and carries opacity', () => {
    const placement = layerPlacement(
      transform({ scale: { x: -1, y: 0.5 }, rotation: 90, opacity: 0.3 }),
      { width: 10, height: 10 },
      frame,
    );
    expect(placement.scaleX).toBe(-1);
    expect(placement.scaleY).toBe(0.5);
    expect(placement.rotation).toBeCloseTo(Math.PI / 2);
    expect(placement.alpha).toBe(0.3);
  });
});

describe('pictureFit', () => {
  const box = { width: 1080, height: 1920 };

  it('stretches upright pixels over the box from its centre', () => {
    expect(pictureFit({ width: 1920, height: 1080 }, 0, false)).toEqual({
      x: 960,
      y: 540,
      width: 1920,
      height: 1080,
      rotation: 0,
      mirrored: false,
    });
  });

  it('swaps the sprite size for quarter turns so the rotated pixels fill the box', () => {
    const fit = pictureFit(box, 90, false);
    expect(fit.width).toBe(1920);
    expect(fit.height).toBe(1080);
    expect(fit.rotation).toBeCloseTo(Math.PI / 2);
    expect(pictureFit(box, 180, false).width).toBe(1080);
  });

  it('mirrors after the rotation by turning the other way', () => {
    expect(pictureFit(box, 90, true).rotation).toBeCloseTo(-Math.PI / 2);
  });
});

describe('outputScale', () => {
  it('maps sequence pixels onto the output on each axis', () => {
    expect(outputScale(frame, { width: 960, height: 540 })).toEqual({ x: 0.5, y: 0.5 });
  });
});

describe('codedCrop', () => {
  const leftHalf = { x: 0, y: 0, width: 0.5, height: 1 };

  it('is the crop itself for upright pixels', () => {
    expect(codedCrop(leftHalf, 0, false)).toEqual(leftHalf);
  });

  it('follows the display rotation back to the decoded pixels', () => {
    // Turned a quarter clockwise, the bottom of the decoded pixels becomes the left side.
    expect(codedCrop(leftHalf, 90, false)).toEqual({ x: 0, y: 0.5, width: 1, height: 0.5 });
    expect(codedCrop(leftHalf, 180, false)).toEqual({ x: 0.5, y: 0, width: 0.5, height: 1 });
    expect(codedCrop(leftHalf, 270, false)).toEqual({ x: 0, y: 0, width: 1, height: 0.5 });
  });

  it('undoes the mirror before the rotation', () => {
    expect(codedCrop(leftHalf, 0, true)).toEqual({ x: 0.5, y: 0, width: 0.5, height: 1 });
    expect(codedCrop(leftHalf, 90, true)).toEqual({ x: 0, y: 0, width: 1, height: 0.5 });
  });
});
