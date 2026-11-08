import type { CaptionStyle, CaptionWord, Flicks } from '@se/protocol';
import { afterEach, describe, expect, it } from 'vitest';

import { presetStyle, type CaptionPresetId } from '../../src/caption-styles';
import { FontRegistry } from '../../src/fonts/font-registry';
import type { RenderScene } from '../../src/preview/frame-renderer';
import { item, rectangle, seconds, sequence, videoTrack } from '../../src/testing/document';
import inter from './fixtures/inter-latin.woff2?url';
import { TEXT_TOLERANCE, expectGolden, litExtent, renderScene, type Rendered } from './golden';

const ms = (value: number) => ((seconds(1) * value) / 1000) as Flicks;

function word(text: string, start: number, end: number, emphasis = false): CaptionWord {
  return { text, start: ms(start), end: ms(end), emphasis };
}

const WORDS = [
  word('Bonjour', 0, 400),
  word('à', 450, 600),
  word('tous', 650, 1000),
  word('et', 1100, 1200),
  word('bienvenue', 1250, 1800),
];

/** A preset drawn with the bundled Inter (the test fixture font), big enough to measure. */
function testStyle(preset: CaptionPresetId, overrides: Partial<CaptionStyle> = {}): CaptionStyle {
  return {
    ...presetStyle(preset, 'style'),
    fontFamily: 'Inter',
    fontSize: 120,
    position: { x: 0.5, y: 0.8 },
    ...overrides,
  };
}

/** A dark frame with the caption over 0–2 s, drawn with `style`. */
function scene(style: CaptionStyle): RenderScene {
  const background = item(
    0 as Flicks,
    seconds(2),
    rectangle('#203040', 10, 10, { coverFrame: true }),
  );
  const caption = item(0 as Flicks, seconds(2), {
    kind: 'caption',
    words: WORDS,
    styleId: 'style',
  });
  return {
    sequence: {
      ...sequence(640, 360, [videoTrack([background]), videoTrack([caption])]),
      captionStyles: [style],
    },
    assets: {},
  };
}

function testFonts(): FontRegistry {
  const registry = new FontRegistry(
    document.fonts,
    (family, source, descriptors) => new FontFace(family, source, descriptors),
  );
  registry.setCatalog([
    {
      family: 'Inter',
      url: inter,
      weightMin: 100,
      weightMax: 900,
      italic: false,
      unicodeRange: null,
      format: 'woff2',
    },
  ]);
  return registry;
}

let rendered: Rendered[] = [];
afterEach(() => {
  for (const entry of rendered) entry.engine.destroy();
  rendered = [];
});

async function render(style: CaptionStyle, time: number): Promise<ImageData> {
  const result = await renderScene(scene(style), ms(time), {}, { fonts: testFonts() });
  rendered.push(result);
  return result.pixels;
}

/** Pixels of a region matching a colour test. */
function count(
  pixels: ImageData,
  region: { x0: number; x1: number; y0: number; y1: number },
  matches: (r: number, g: number, b: number) => boolean,
): number {
  let total = 0;
  for (let y = region.y0; y < region.y1; y += 1) {
    for (let x = region.x0; x < region.x1; x += 1) {
      const offset = (y * pixels.width + x) * 4;
      const [r, g, b] = [
        pixels.data[offset] ?? 0,
        pixels.data[offset + 1] ?? 0,
        pixels.data[offset + 2] ?? 0,
      ];
      if (matches(r, g, b)) total += 1;
    }
  }
  return total;
}

const yellow = (r: number, g: number, b: number) => r > 200 && g > 160 && b < 90;
const white = (r: number, g: number, b: number) => r > 200 && g > 200 && b > 200;
/** The band of the caption's single line (bottom at 0.8 × 360 = 288 px). */
const BAND = { y0: 240, y1: 290 };

describe('caption golden frames', () => {
  it('karaoke: words spoken so far are filled in, the rest stays white', async () => {
    // At 0.7 s « tous » is spoken: « Bonjour à tous » yellow, « et bienvenue » white.
    const pixels = await render(testStyle('karaoke'), 700);
    await expectGolden('caption-karaoke-0_7s', pixels, TEXT_TOLERANCE);
    const left = { ...BAND, x0: 0, x1: 320 };
    const right = { ...BAND, x0: 420, x1: 640 };
    expect(count(pixels, left, yellow)).toBeGreaterThan(200);
    expect(count(pixels, right, yellow)).toBe(0);
    expect(count(pixels, right, white)).toBeGreaterThan(200);
  });

  it('the highlight follows the speech from word to word', async () => {
    const style = testStyle('karaoke', { highlightSpoken: false });
    const early = await render(style, 100);
    const late = await render(style, 1300);
    const firstWord = { ...BAND, x0: 0, x1: 200 };
    const lastWord = { ...BAND, x0: 420, x1: 640 };
    expect(count(early, firstWord, yellow)).toBeGreaterThan(100);
    expect(count(early, lastWord, yellow)).toBe(0);
    expect(count(late, firstWord, yellow)).toBe(0);
    expect(count(late, lastWord, yellow)).toBeGreaterThan(100);
  });

  it('tiktok bold: one upper-case line popping on the active word', async () => {
    // 90 ms after « bienvenue » starts it is at its biggest.
    const pixels = await render(testStyle('tiktokBold', { maxCharsPerLine: 18 }), 1340);
    await expectGolden('caption-tiktok-bold-pop', pixels, TEXT_TOLERANCE);
    const still = await render(testStyle('tiktokBold', { maxCharsPerLine: 18 }), 1700);
    const popped = litExtent(pixels, { ...BAND, x0: 0, x1: 640 });
    const rest = litExtent(still, { ...BAND, x0: 0, x1: 640 });
    expect(popped).not.toBeNull();
    expect(rest).not.toBeNull();
    // Only the last page (« BIENVENUE ») shows; popping makes it wider than at rest.
    expect((popped?.right ?? 0) - (popped?.left ?? 0)).toBeGreaterThan(
      (rest?.right ?? 0) - (rest?.left ?? 0),
    );
  });

  it('box: each line sits on its own plate', async () => {
    const pixels = await render(testStyle('box', { maxCharsPerLine: 16 }), 700);
    await expectGolden('caption-box-per-line', pixels, TEXT_TOLERANCE);
    const plate = (r: number, g: number, b: number) => r < 40 && g < 40 && b < 40;
    // Two lines: « Bonjour à tous » above « et bienvenue », each on a near-black plate.
    expect(count(pixels, { x0: 0, x1: 640, y0: 200, y1: 240 }, plate)).toBeGreaterThan(500);
    expect(count(pixels, { x0: 0, x1: 640, y0: 250, y1: 288 }, plate)).toBeGreaterThan(500);
  });

  it('reveal: words appear as they are spoken without moving', async () => {
    const style = testStyle('minimal', { animation: 'revealPerWord', fontSize: 120 });
    const partial = await render(style, 420);
    const full = await render(style, 1900);
    await expectGolden('caption-reveal-partial', partial, TEXT_TOLERANCE);
    const shown = litExtent(partial, { ...BAND, x0: 0, x1: 640 });
    const all = litExtent(full, { ...BAND, x0: 0, x1: 640 });
    expect(shown?.left).toBe(all?.left);
    expect(shown?.right ?? 0).toBeLessThan((all?.right ?? 0) - 100);
  });
});
