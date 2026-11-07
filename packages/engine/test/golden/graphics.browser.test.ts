import type { Animatable, Flicks } from '@se/protocol';
import { afterEach, describe, expect, it } from 'vitest';

import { FontRegistry } from '../../src/fonts/font-registry';
import type { RenderScene } from '../../src/preview/frame-renderer';
import {
  ellipse,
  item,
  line,
  rectangle,
  seconds,
  sequence,
  still,
  stroke,
  text,
  textStyle,
  videoTrack,
} from '../../src/testing/document';
import interItalic from './fixtures/inter-latin-italic.woff2?url';
import inter from './fixtures/inter-latin.woff2?url';
import {
  TEXT_TOLERANCE,
  expectGolden,
  litExtent,
  luminanceAt,
  renderScene,
  type Rendered,
  type RenderOptions,
} from './golden';

/** Design pixels per sequence pixel of a 640 × 360 frame (1080 design px = 360 px). */
const DESIGN = 3;
const ALL = seconds(2);
const at = (kind: Parameters<typeof item>[2]) => item(0 as Flicks, ALL, kind);
const scene = (...layers: Parameters<typeof item>[2][]): RenderScene => ({
  sequence: sequence(
    640,
    360,
    layers.map((layer) => videoTrack([at(layer)])),
  ),
  assets: {},
});

/** The bundled Inter (upright and italic), the same file on every platform. */
function testFonts(): FontRegistry {
  const registry = new FontRegistry(
    document.fonts,
    (family, source, descriptors) => new FontFace(family, source, descriptors),
  );
  const face = (url: string, italic: boolean) => ({
    family: 'Inter',
    url,
    weightMin: 100,
    weightMax: 900,
    italic,
    unicodeRange: null,
    format: 'woff2',
  });
  registry.setCatalog([face(inter, false), face(interItalic, true)]);
  return registry;
}

let rendered: Rendered[] = [];
afterEach(() => {
  for (const entry of rendered) entry.engine.destroy();
  rendered = [];
});

async function render(
  target: RenderScene,
  time: Flicks = 0 as Flicks,
  options: RenderOptions = {},
) {
  const result = await renderScene(target, time, {}, options);
  rendered.push(result);
  return result.pixels;
}

const keyframes = <T>(points: readonly [number, T][]): Animatable<T> => ({
  kind: 'keyframed',
  keyframes: points.map(([time, value]) => ({
    time: seconds(time),
    value,
    easing: { kind: 'preset', name: 'easeOut' },
  })),
});

describe('graphics golden frames', () => {
  it('draws outlines, rounded corners, lines and a frame-covering plate', async () => {
    const pixels = await render(
      scene(
        rectangle('#1e272e', 10, 10, { coverFrame: true }),
        rectangle('#c0392b', 120 * DESIGN, 70 * DESIGN, {
          stroke: stroke('#f1c40f', 4 * DESIGN),
          transform: { position: { x: 0.25, y: 0.3 } },
        }),
        rectangle('#2e86de', 120 * DESIGN, 70 * DESIGN, {
          cornerRadius: 20 * DESIGN,
          stroke: stroke('#ffffff', 2 * DESIGN),
          transform: { position: { x: 0.7, y: 0.3 } },
        }),
        ellipse(
          '#27ae60',
          80 * DESIGN,
          50 * DESIGN,
          { position: { x: 0.3, y: 0.75 } },
          {
            stroke: stroke('#000000', 3 * DESIGN),
          },
        ),
        line('#ecf0f1', 200 * DESIGN, 6 * DESIGN, {
          transform: { position: { x: 0.72, y: 0.75 }, rotation: -30 },
        }),
      ),
    );
    await expectGolden('shapes-strokes-lines', pixels);
  });

  it('rotates and scales around the anchor, and blurs', async () => {
    const pixels = await render(
      scene(
        rectangle('#8e44ad', 100 * DESIGN, 60 * DESIGN, {
          transform: { position: { x: 0.25, y: 0.5 }, anchor: { x: 0, y: 0 }, rotation: 45 },
        }),
        rectangle('#e67e22', 100 * DESIGN, 60 * DESIGN, {
          transform: { position: { x: 0.55, y: 0.5 }, scale: { x: 1.5, y: 0.5 }, rotation: -20 },
        }),
        ellipse('#16a085', 90 * DESIGN, 90 * DESIGN, {
          position: { x: 0.82, y: 0.5 },
          blur: 8 * DESIGN,
        }),
      ),
    );
    await expectGolden('rotation-anchor-blur', pixels);
  });

  it('animates opacity and position along eased keyframes (a slide-in midway)', async () => {
    const slide = rectangle('#f1c40f', 160 * DESIGN, 90 * DESIGN, {
      transform: {
        position: keyframes([
          [0, { x: 0.4, y: 0.5 }],
          [1, { x: 0.5, y: 0.5 }],
        ]),
        opacity: keyframes([
          [0, 0],
          [0.6, 1],
        ]),
      },
    });
    const pixels = await render(scene(slide), seconds(0.3));
    await expectGolden('slide-in-midway', pixels);
  });

  it('draws text styles with the bundled font', async () => {
    const fonts = testFonts();
    const pixels = await render(
      scene(
        text('Contour', {
          style: { fontSize: still((64 * DESIGN) / 2), stroke: stroke('#e74c3c', 3 * DESIGN) },
          transform: { position: { x: 0.25, y: 0.2 } },
        }),
        text('Ombre', {
          style: {
            fontSize: still((64 * DESIGN) / 2),
            shadow: {
              enabled: true,
              color: still('#3498db'),
              blur: still(2 * DESIGN),
              offset: still({ x: 4 * DESIGN, y: 4 * DESIGN }),
            },
            color: still('#f1c40f'),
          },
          transform: { position: { x: 0.75, y: 0.2 } },
        }),
        text('Plaque', {
          style: textStyle({
            fontSize: still((56 * DESIGN) / 2),
            background: {
              enabled: true,
              color: still('#2980b9'),
              padding: { x: 20 * DESIGN, y: 8 * DESIGN },
              cornerRadius: 10 * DESIGN,
            },
          }),
          transform: { position: { x: 0.25, y: 0.55 } },
        }),
        text('espacé', {
          style: {
            fontSize: still((48 * DESIGN) / 2),
            textCase: 'uppercase',
            letterSpacing: still(8 * DESIGN),
            fontWeight: 800,
            italic: true,
          },
          transform: { position: { x: 0.75, y: 0.55 } },
        }),
        text('Aligné à gauche sur deux lignes', {
          style: {
            fontSize: still((36 * DESIGN) / 2),
            align: 'left',
            maxWidth: 260 * DESIGN,
            fontWeight: 400,
          },
          transform: { position: { x: 0.05, y: 0.75 }, anchor: { x: 0, y: 0 } },
        }),
        text('Machine à écrire', {
          style: { fontSize: still((40 * DESIGN) / 2) },
          reveal: 0.5,
          transform: { position: { x: 0.75, y: 0.85 } },
        }),
      ),
      0 as Flicks,
      { fonts },
    );
    expect(document.fonts.check('600 20px "Inter"')).toBe(true);
    await expectGolden('text-styles', pixels, TEXT_TOLERANCE);
    // The outline is red around white glyphs.
    let red = 0;
    for (let y = 40; y < 105; y += 1) {
      for (let x = 40; x < 280; x += 1) {
        const offset = (y * 640 + x) * 4;
        if ((pixels.data[offset] ?? 0) > 180 && (pixels.data[offset + 1] ?? 0) < 120) red += 1;
      }
    }
    expect(red).toBeGreaterThan(200);
    // The plate is blue around its text.
    const plate = (y: number, x: number) => pixels.data[(y * 640 + x) * 4 + 2] ?? 0;
    expect(plate(Math.round(0.55 * 360), Math.round(0.25 * 640) - 55)).toBeGreaterThan(150);
    // The shadow is blue, below and right of the yellow glyphs.
    let blue = 0;
    for (let y = 40; y < 105; y += 1) {
      for (let x = 400; x < 600; x += 1) {
        const offset = (y * 640 + x) * 4;
        if ((pixels.data[offset + 2] ?? 0) > 150 && (pixels.data[offset] ?? 0) < 120) blue += 1;
      }
    }
    expect(blue).toBeGreaterThan(100);
    // The yellow « Ombre » is yellow.
    let yellow = 0;
    for (let y = 40; y < 105; y += 1) {
      for (let x = 400; x < 600; x += 1) {
        const offset = (y * 640 + x) * 4;
        const [r, g, b] = [
          pixels.data[offset] ?? 0,
          pixels.data[offset + 1] ?? 0,
          pixels.data[offset + 2] ?? 0,
        ];
        if (r > 180 && g > 150 && b < 90) yellow += 1;
      }
    }
    expect(yellow).toBeGreaterThan(200);
    // « ESPACÉ » (24 px, 8 px of letter spacing) is spread wider than its glyphs alone.
    const spaced = litExtent(pixels, { x0: 320, x1: 640, y0: 185, y1: 212 });
    expect((spaced?.right ?? 0) - (spaced?.left ?? 0)).toBeGreaterThan(120);
    // The wrapped paragraph starts on its anchor, 5 % from the left edge (x = 32).
    const paragraph = litExtent(pixels, { x0: 0, x1: 320, y0: 268, y1: 318 });
    expect(paragraph?.left).toBeGreaterThanOrEqual(31);
    expect(paragraph?.left).toBeLessThanOrEqual(36);
    // Half of « Machine à écrire » is typed: its right part is still dark.
    expect(luminanceAt(pixels, 0.75 * 640 + 80, 0.85 * 360)).toBeLessThan(20);
  });
});
