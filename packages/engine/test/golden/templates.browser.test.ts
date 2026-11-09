/**
 * Motion templates through the real engine (prompt 24): an instance of the instructions' lower
 * third drawn by its sandboxed Worker and composited like any layer (golden frame), the same
 * frame drawn identically by a second engine, and a failing template drawn as the error picture
 * with its issue.
 */
import type { Flicks, MotionTemplate, TemplateParamValue } from '@se/protocol';
import { afterEach, describe, expect, it } from 'vitest';

import type { RenderScene } from '../../src/preview/frame-renderer';
import {
  item,
  rectangle,
  seconds,
  sequence,
  transform,
  videoTrack,
} from '../../src/testing/document';
import { catalogFontSource } from '../../src/templates/font-faces';
import { TemplateRuntime } from '../../src/templates/template-runtime';
import lowerThird from '../../../../crates/se-app/assets/agent-instructions/templates/lower-third.ts?raw';
import inter from './fixtures/inter-latin.woff2?url';
import { TEXT_TOLERANCE, expectGolden, renderScene, type Rendered } from './golden';

const ms = (value: number) => ((seconds(1) * value) / 1000) as Flicks;

function template(source: string, id = 'template'): MotionTemplate {
  return {
    id,
    name: 'Lower third',
    description: '',
    version: 1,
    source,
    params: [
      {
        name: 'name',
        label: 'Nom',
        description: null,
        brand: null,
        spec: { kind: 'string', default: 'Jane Doe', multiline: false, maxLength: 60 },
      },
      {
        name: 'role',
        label: 'Titre',
        description: null,
        brand: null,
        spec: { kind: 'string', default: 'Réalisatrice', multiline: false, maxLength: 80 },
      },
      {
        name: 'accent',
        label: 'Accent',
        description: null,
        brand: null,
        spec: { kind: 'color', default: '#ffcc00' },
      },
      {
        name: 'textColor',
        label: 'Texte',
        description: null,
        brand: null,
        spec: { kind: 'color', default: '#ffffff' },
      },
      {
        name: 'font',
        label: 'Police',
        description: null,
        brand: null,
        spec: { kind: 'font', default: 'Inter' },
      },
      {
        name: 'slide',
        label: 'Glissement',
        description: null,
        brand: null,
        spec: { kind: 'number', default: 80, minimum: 0, maximum: 400, step: null },
      },
    ],
    defaultDuration: seconds(5),
    origin: { kind: 'user' },
    category: 'other',
    library: null,
  };
}

/** A dark frame with an instance of `motion` over 0–5 s. */
function scene(
  motion: MotionTemplate,
  values: Record<string, TemplateParamValue> = {},
): RenderScene {
  const background = item(
    0 as Flicks,
    seconds(5),
    rectangle('#203040', 10, 10, { coverFrame: true }),
  );
  const instance = item(0 as Flicks, seconds(5), {
    kind: 'motionTemplate',
    templateId: motion.id,
    params: values,
    transform: transform(),
  });
  return {
    sequence: {
      ...sequence(640, 360, [videoTrack([background]), videoTrack([instance])]),
      templates: [motion],
    },
    assets: {},
  };
}

function runtime(): TemplateRuntime {
  return new TemplateRuntime({
    fonts: catalogFontSource([
      {
        family: 'Inter',
        url: inter,
        weightMin: 100,
        weightMax: 900,
        italic: false,
        unicodeRange: null,
        format: 'woff2',
      },
    ]),
  });
}

let rendered: Rendered[] = [];
let runtimes: TemplateRuntime[] = [];
afterEach(() => {
  for (const entry of rendered) entry.engine.destroy();
  for (const entry of runtimes) entry.destroy();
  rendered = [];
  runtimes = [];
});

async function draw(motion: MotionTemplate, time: Flicks, values = {}): Promise<Rendered> {
  const templates = runtime();
  runtimes.push(templates);
  const frame = await renderScene(scene(motion, values), time, {}, { templates });
  rendered.push(frame);
  return frame;
}

function pixelAt(pixels: ImageData, x: number, y: number): readonly number[] {
  const offset = (y * pixels.width + x) * 4;
  return [...pixels.data.slice(offset, offset + 3)];
}

describe('motion templates in the engine', () => {
  it('draws the lower third with its values (golden frame)', async () => {
    const { pixels, engine } = await draw(template(lowerThird), ms(1500), {
      name: { kind: 'text', value: 'Ada Lovelace' },
    });
    expect(engine.getState().templateIssues).toEqual([]);
    // The accent bar: design (86, 804)–(98, 974) of 1920 × 1080, a third of it at 640 × 360.
    const [red, green, blue] = pixelAt(pixels, 30, 290);
    expect(red).toBeGreaterThan(200);
    expect(green).toBeGreaterThan(160);
    expect(blue).toBeLessThan(80);
    await expectGolden('template-lower-third-1_5s', pixels, TEXT_TOLERANCE);
  });

  it('draws the same frame again in another engine (determinism)', async () => {
    const first = await draw(template(lowerThird), ms(800));
    const second = await draw(template(lowerThird), ms(800));
    expect([...second.pixels.data]).toEqual([...first.pixels.data]);
  });

  it('draws the error picture for a failing template and reports it', async () => {
    const broken = template(
      "export const meta = { name: 'x', defaultDuration: 5 };\nexport const params = {};\nexport function render(ctx) {\n  ctx.fillStyle = undefinedName;\n}\n",
      'broken',
    );
    const { engine, pixels } = await draw(broken, ms(1000));
    const [issue] = engine.getState().templateIssues;
    expect(issue).toMatchObject({ templateId: 'broken', line: 4 });
    expect(issue?.message).toContain('undefinedName is not defined');
    // The error picture's red border, over the dark background.
    const [red] = pixelAt(pixels, 0, 180);
    expect(red).toBeGreaterThan(150);
  });
});
