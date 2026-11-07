import { describe, expect, it } from 'vitest';

import { fillCrop, reframeScale, reframedPicture } from './reframe';

const landscape = { width: 1920, height: 1080 };
const vertical = { width: 1080, height: 1920 };

describe('fillCrop', () => {
  it('keeps the focus at the centre of a window of the frame aspect', () => {
    const crop = fillCrop(landscape, vertical, { x: 0.5, y: 0.5 });
    expect(crop.width).toBeCloseTo(1080 / 3413.333, 6);
    expect(crop.height).toBe(1);
    expect(crop.x + crop.width / 2).toBeCloseTo(0.5, 9);
    expect(crop.y).toBe(0);
  });

  it('never leaves the picture', () => {
    const left = fillCrop(landscape, vertical, { x: 0, y: 0 });
    expect(left.x).toBe(0);
    const right = fillCrop(landscape, vertical, { x: 1, y: 1 });
    expect(right.x + right.width).toBeCloseTo(1, 9);
    expect(right.y).toBe(0);
  });

  it('crops vertically when the picture is taller than the frame', () => {
    const crop = fillCrop(vertical, landscape, { x: 0.5, y: 0.2 });
    expect(crop.width).toBe(1);
    expect(crop.height).toBeCloseTo(1080 / 3413.333, 6);
    expect(crop.y).toBeCloseTo(0.2 - crop.height / 2, 9);
  });

  it('shows the whole picture when it has the frame aspect', () => {
    expect(fillCrop(landscape, { width: 3840, height: 2160 }, { x: 0.1, y: 0.9 })).toEqual({
      x: 0,
      y: 0,
      width: 1,
      height: 1,
    });
  });
});

describe('reframedPicture', () => {
  const centre = { x: 0.5, y: 0.5 };

  it('fits inside, fills the frame or keeps the natural size in design pixels', () => {
    expect(reframedPicture(landscape, vertical, 'fit', centre, 1)).toEqual({
      box: { width: 1080, height: 607.5 },
      crop: null,
    });
    expect(reframedPicture(landscape, vertical, 'fill', centre, 1).box).toEqual(vertical);
    expect(reframedPicture(landscape, vertical, 'tracked', centre, 1).box).toEqual(vertical);
    expect(reframedPicture(landscape, { width: 3840, height: 2160 }, 'custom', centre, 2)).toEqual({
      box: { width: 3840, height: 2160 },
      crop: null,
    });
  });

  it('fills the frame exactly when the picture size is unknown', () => {
    expect(reframedPicture(null, vertical, 'custom', centre, 1)).toEqual({
      box: vertical,
      crop: null,
    });
  });
});

describe('reframeScale', () => {
  it('gives sequence pixels per picture pixel in each mode', () => {
    expect(reframeScale(landscape, vertical, 'fit', 1)).toBeCloseTo(0.5625, 9);
    expect(reframeScale(landscape, vertical, 'fill', 1)).toBeCloseTo(1.7778, 4);
    expect(reframeScale(landscape, vertical, 'custom', 1)).toBe(1);
    expect(reframeScale(null, vertical, 'fit', 1)).toBeNull();
  });
});
