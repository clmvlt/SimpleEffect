// Invariants of the palette: every colour is displayable in sRGB, and text tokens reach WCAG AA
// (4.5:1) on every surface they are used on, in every accent shade. Tune the OKLCH values in
// `colors.css` / `accents.css` if this fails; never lower the thresholds.
import { describe, expect, it } from 'vitest';

import accentsCss from './accents.css?raw';
import {
  compositeOver,
  contrastRatio,
  isInSrgbGamut,
  oklchToSrgb,
  parseOklch,
  type Srgb,
} from './color-math';
import colorsCss from './colors.css?raw';
import { readAccentVariants, readTokens } from './token-css';

const AA_TEXT = 4.5;
const AA_NON_TEXT = 3;

const tokens = readTokens(colorsCss);

/** Resolves `color-<name>` to sRGB; translucent tokens are painted over `under` (an opaque token). */
function colour(
  name: string,
  palette: ReadonlyMap<string, string> = tokens,
  under = 'panel',
): Srgb {
  const value = palette.get(`color-${name}`) ?? tokens.get(`color-${name}`);
  const parsed = value === undefined ? null : parseOklch(value);
  if (parsed === null) {
    throw new Error(`Token --color-${name} is missing or is not an oklch() value.`);
  }
  return parsed.alpha < 1 ? compositeOver(parsed, colour(under, palette)) : oklchToSrgb(parsed);
}

function expectContrast(foreground: Srgb, background: Srgb, minimum: number): void {
  expect(contrastRatio(foreground, background)).toBeGreaterThanOrEqual(minimum);
}

const SURFACES = ['bg', 'panel', 'panel-raised', 'overlay', 'input'];

describe('text tokens', () => {
  it.each(['text', 'text-muted', 'text-subtle'])('%s is AA on every surface', (text) => {
    for (const surface of SURFACES) {
      expectContrast(colour(text), colour(surface), AA_TEXT);
    }
  });

  it.each(['control', 'control-hover'])('primary and muted text are AA on %s', (surface) => {
    expectContrast(colour('text'), colour(surface), AA_TEXT);
    expectContrast(colour('text-muted'), colour(surface), AA_TEXT);
  });

  it('text-inverted is AA on primary text (light chips)', () => {
    expectContrast(colour('text-inverted'), colour('text'), AA_TEXT);
  });

  it.each(['success', 'warning', 'danger', 'info'])('%s reads as text on panels', (state) => {
    for (const surface of SURFACES) {
      expectContrast(colour(state), colour(surface), AA_TEXT);
    }
  });

  it('danger buttons keep AA labels', () => {
    expectContrast(colour('danger-foreground'), colour('danger-solid'), AA_TEXT);
    expectContrast(colour('danger-foreground'), colour('danger-solid-hover'), AA_TEXT);
  });

  it.each(['item-video', 'item-audio', 'item-graphic', 'item-caption', 'ai'])(
    '%s badges keep AA text on their muted fill',
    (kind) => {
      expectContrast(colour('text'), colour(`${kind}-muted`), AA_TEXT);
      expectContrast(colour(`${kind}-strong`), colour('panel'), AA_TEXT);
    },
  );
});

const accents: [string, ReadonlyMap<string, string>][] = [
  ['default', tokens],
  ...readAccentVariants(accentsCss),
];

describe.each(accents)('accent %s', (_name, palette) => {
  it('keeps AA labels on filled accent controls', () => {
    expectContrast(colour('accent-foreground', palette), colour('accent', palette), AA_TEXT);
    expectContrast(colour('accent-foreground', palette), colour('accent-hover', palette), AA_TEXT);
  });

  it('keeps AA text on selected rows', () => {
    expectContrast(colour('text', palette), colour('accent-muted', palette, 'panel'), AA_TEXT);
  });

  it('has a focus ring visible on every surface', () => {
    for (const surface of SURFACES) {
      expectContrast(colour('focus-ring', palette), colour(surface), AA_NON_TEXT);
    }
  });
});

describe('accent shades', () => {
  it('are the three shades presented in Storybook', () => {
    expect(accents.map(([name]) => name)).toEqual(['default', 'lemon', 'yellow', 'gold']);
  });

  it('use the chosen shade (yellow) as the default accent', () => {
    const yellow = new Map(readAccentVariants(accentsCss)).get('yellow');
    for (const [name, value] of yellow ?? []) {
      expect(tokens.get(name)).toBe(value);
    }
    expect(yellow?.size).toBe(5);
  });
});

it.each([
  ...tokens,
  ...[...readAccentVariants(accentsCss).values()].flatMap((shade) => [...shade]),
])('%s is inside the sRGB gamut', (_name, value) => {
  const colour = parseOklch(value);
  expect(colour === null || isInSrgbGamut(colour)).toBe(true);
});
