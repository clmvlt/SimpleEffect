import { describe, expect, it } from 'vitest';

import {
  compositeOver,
  contrastRatio,
  isInSrgbGamut,
  oklchToSrgb,
  parseHex,
  parseOklch,
  srgbToOklch,
  toHex,
} from './color-math';
import { parseTokenDeclarations, readAccentVariants } from './token-css';

describe('parseOklch', () => {
  it('reads lightness, chroma, hue and an optional alpha', () => {
    expect(parseOklch('oklch(0.5 0.1 250)')).toEqual({
      lightness: 0.5,
      chroma: 0.1,
      hue: 250,
      alpha: 1,
    });
    expect(parseOklch('oklch(60% 0.2 30deg / 40%)')).toEqual({
      lightness: 0.6,
      chroma: 0.2,
      hue: 30,
      alpha: 0.4,
    });
  });

  it('rejects anything else', () => {
    expect(parseOklch('var(--color-text)')).toBeNull();
  });
});

describe('oklchToSrgb', () => {
  it('maps the extremes to black and white', () => {
    expect(toHex(oklchToSrgb({ lightness: 0, chroma: 0, hue: 0, alpha: 1 }))).toBe('#000000');
    expect(toHex(oklchToSrgb({ lightness: 1, chroma: 0, hue: 0, alpha: 1 }))).toBe('#ffffff');
  });

  it('matches a reference conversion (CSS Color 4 sample)', () => {
    // oklch(0.628 0.2577 29.23) is sRGB red.
    expect(toHex(oklchToSrgb({ lightness: 0.628, chroma: 0.2577, hue: 29.23, alpha: 1 }))).toBe(
      '#ff0000',
    );
  });
});

describe('isInSrgbGamut', () => {
  it('accepts displayable colours and rejects over-saturated ones', () => {
    expect(isInSrgbGamut({ lightness: 0.628, chroma: 0.2577, hue: 29.23, alpha: 1 })).toBe(true);
    expect(isInSrgbGamut({ lightness: 0.9, chroma: 0.3, hue: 140, alpha: 1 })).toBe(false);
  });
});

describe('contrastRatio', () => {
  const black = { red: 0, green: 0, blue: 0 };
  const white = { red: 1, green: 1, blue: 1 };

  it('is 21 between black and white and symmetric', () => {
    expect(contrastRatio(black, white)).toBeCloseTo(21);
    expect(contrastRatio(white, black)).toBeCloseTo(21);
  });

  it('composites translucent colours before measuring', () => {
    const halfWhite = { lightness: 1, chroma: 0, hue: 0, alpha: 0.5 };
    const grey = compositeOver(halfWhite, black);
    expect(grey.red).toBeCloseTo(0.5);
    expect(grey.blue).toBeCloseTo(0.5);
  });
});

describe('srgbToOklch', () => {
  it('inverts oklchToSrgb for colours inside the gamut', () => {
    const colour = { lightness: 0.7, chroma: 0.12, hue: 250, alpha: 1 };
    const back = srgbToOklch(oklchToSrgb(colour));
    expect(back.lightness).toBeCloseTo(0.7, 4);
    expect(back.chroma).toBeCloseTo(0.12, 4);
    expect(back.hue).toBeCloseTo(250, 2);
  });

  it('reports greys with zero chroma and hue', () => {
    expect(srgbToOklch({ red: 0.5, green: 0.5, blue: 0.5 })).toMatchObject({ chroma: 0, hue: 0 });
  });
});

describe('parseHex', () => {
  it('reads short and long forms, with or without #', () => {
    expect(parseHex('#fff')).toEqual({ red: 1, green: 1, blue: 1 });
    expect(parseHex('FF0000')).toEqual({ red: 1, green: 0, blue: 0 });
    expect(toHex(parseHex(' #33AAff ') ?? { red: 0, green: 0, blue: 0 })).toBe('#33aaff');
  });

  it('rejects anything else', () => {
    expect(parseHex('#ff00')).toBeNull();
    expect(parseHex('red')).toBeNull();
    expect(parseHex('#gg0000')).toBeNull();
  });
});

describe('token stylesheet parsing', () => {
  it('lists declarations and skips namespace resets and comments', () => {
    const css =
      '@theme { --color-*: initial; /* --color-gone: red; */ --color-a: oklch(0.5 0 0); }';
    expect(parseTokenDeclarations(css)).toEqual([{ name: 'color-a', value: 'oklch(0.5 0 0)' }]);
  });

  it('splits accent variants', () => {
    const css =
      "[data-accent='blue'] { --color-accent: x; } [data-accent='red'] { --color-accent: y; }";
    const variants = readAccentVariants(css);
    expect([...variants.keys()]).toEqual(['blue', 'red']);
    expect(variants.get('red')?.get('color-accent')).toBe('y');
  });
});
