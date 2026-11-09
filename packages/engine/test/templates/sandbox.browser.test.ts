/**
 * The motion template sandbox in a real browser (prompt 24): checks of valid modules, forbidden
 * globals refused with their line, endless loops stopped, the frame budget, determinism, and
 * compile and export problems.
 */
import type { TemplateCheckOutcome, TemplateProblem } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { checkTemplate } from '../../src/templates/check-template';
import { catalogFontSource } from '../../src/templates/font-faces';
import { blobWorkerFactory } from '../../src/templates/template-worker';
import oswald from '../../../../crates/se-fonts/fonts/oswald/oswald-latin-wght-normal.woff2?url';
import animatedTitle from '../../../../crates/se-app/assets/agent-instructions/templates/animated-title.ts?raw';
import callToAction from '../../../../crates/se-app/assets/agent-instructions/templates/call-to-action.ts?raw';
import lowerThird from '../../../../crates/se-app/assets/agent-instructions/templates/lower-third.ts?raw';
import progressBar from '../../../../crates/se-app/assets/agent-instructions/templates/progress-bar.ts?raw';

const OPTIONS = { width: 1920, height: 1080, pictureWidth: 320, pictureHeight: 180 };

/** A module whose `render` body starts at line 4. */
function moduleWith(body: string, before = ''): string {
  return [
    "export const meta = { name: 'Test', defaultDuration: 2 };",
    "export const params = { color: { type: 'string', format: 'color', default: '#ff0000' } };",
    before + 'export function render(ctx, time, p) {',
    body,
    '}',
    '',
  ].join('\n');
}

function problems(outcome: TemplateCheckOutcome): readonly TemplateProblem[] {
  if (outcome.kind !== 'failed') throw new Error('the module was expected to be refused');
  return outcome.problems;
}

async function decode(base64: string): Promise<ImageData> {
  const bytes = Uint8Array.from(atob(base64), (character) => character.charCodeAt(0));
  const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/jpeg' }));
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  const context = canvas.getContext('2d');
  if (context === null) throw new Error('no 2D context');
  context.drawImage(bitmap, 0, 0);
  return context.getImageData(0, 0, bitmap.width, bitmap.height);
}

function redAt(pixels: ImageData, x: number, y: number): number {
  return pixels.data[(y * pixels.width + x) * 4] ?? 0;
}

describe('template checks', () => {
  it('passes a valid module with its declarations and three pictures', async () => {
    const outcome = await checkTemplate(
      moduleWith('ctx.fillStyle = p.color;\nctx.fillRect(0, 0, ctx.width / 2, ctx.height);'),
      OPTIONS,
    );
    expect(outcome.kind).toBe('passed');
    if (outcome.kind !== 'passed') return;
    expect(JSON.parse(outcome.metaJson)).toEqual({ name: 'Test', defaultDuration: 2 });
    expect(JSON.parse(outcome.paramsJson)).toEqual([
      ['color', { type: 'string', format: 'color', default: '#ff0000' }],
    ]);
    expect(outcome.frames).toHaveLength(3);
    expect(outcome.slowestFrameMs).toBeLessThan(50);
    const pixels = await decode(outcome.frames[1]?.jpegBase64 ?? '');
    expect([pixels.width, pixels.height]).toEqual([320, 180]);
    expect(redAt(pixels, 40, 90)).toBeGreaterThan(200);
    expect(redAt(pixels, 280, 90)).toBeLessThan(40);
  });

  it('checks again in a Worker given a family first asked for while rendering', async () => {
    // Electron's Chromium keeps the fallback for a family a Worker used before its faces
    // arrived: a family named only in the code (not a font parameter) needs a second Worker.
    let started = 0;
    const outcome = await checkTemplate(
      moduleWith("ctx.text('Oswald', 100, 100, { family: 'Oswald', size: 80, color: p.color });"),
      {
        ...OPTIONS,
        fonts: catalogFontSource([
          {
            family: 'Oswald',
            url: oswald,
            weightMin: 200,
            weightMax: 700,
            italic: false,
            unicodeRange: null,
            format: 'woff2',
          },
        ]),
        createWorker: (source) => {
          started += 1;
          return blobWorkerFactory(source);
        },
      },
    );
    expect(outcome.kind).toBe('passed');
    expect(started).toBe(2);
  });

  it.each([
    ["fetch('http://127.0.0.1:1/');", 'fetch is not available'],
    ['Date.now();', 'Date is not available'],
    ['Math.random();', 'Math.random is not available'],
    ['setTimeout(() => undefined, 0);', 'setTimeout is not available'],
    ["importScripts('x.js');", 'importScripts is not a function'],
    ["indexedDB.open('x');", 'indexedDB is not available'],
    ['new OffscreenCanvas(1, 1);', 'OffscreenCanvas is not available'],
    ['navigator.userAgent;', 'navigator is not available'],
    ['new XMLHttpRequest();', 'XMLHttpRequest is not available'],
    ["eval('1');", 'eval is not available'],
    ["new Function('return this')();", 'Function is not available'],
    ["(() => undefined).constructor('return this')();", 'Function constructor is not available'],
    ["self.postMessage('forged');", 'Cannot read properties of undefined'],
    ['globalThis.fetch;', 'Cannot read properties of undefined'],
  ])('refuses %s with its line', async (statement, message) => {
    const [problem] = problems(await checkTemplate(moduleWith(statement), OPTIONS));
    expect(problem?.stage).toBe('render');
    expect(problem?.line).toBe(4);
    expect(problem?.message).toContain(message);
    expect(problem?.time).toBe(0);
  });

  it('stops an endless loop and says so', async () => {
    const started = performance.now();
    const [problem] = problems(await checkTemplate(moduleWith('for (;;) {}'), OPTIONS));
    expect(problem?.stage).toBe('budget');
    expect(problem?.message).toContain('endless loop');
    expect(performance.now() - started).toBeLessThan(5000);
  });

  it('stops an endless loop at the top level', async () => {
    const [problem] = problems(await checkTemplate(moduleWith('', 'for (;;) {}\n'), OPTIONS));
    expect(problem?.stage).toBe('budget');
  });

  it('refuses frames over the budget with their instant', async () => {
    const slow =
      'let total = 0;\nfor (let i = 0; i < 2e7; i += 1) total += i;\nctx.fillRect(0, 0, total % 2, 1);';
    const found = problems(await checkTemplate(moduleWith(slow), { ...OPTIONS, budgetMs: 1 }));
    expect(found[0]?.stage).toBe('budget');
    expect(found[0]?.message).toContain('the budget is 1 ms');
    expect(found.map((problem) => problem.time)).not.toContain(null);
  });

  it('refuses state kept between renders', async () => {
    const [problem] = problems(
      await checkTemplate(
        moduleWith('calls += 1;\nctx.fillRect(0, 0, calls * 20, 20);', 'let calls = 0;\n'),
        OPTIONS,
      ),
    );
    expect(problem?.stage).toBe('determinism');
    expect(problem?.message).toContain('twice gave different pictures');
  });

  it('names missing exports and top-level exceptions', async () => {
    const missing = problems(await checkTemplate("export const meta = { name: 'x' };", OPTIONS));
    expect(missing.map((problem) => problem.stage)).toEqual(['exports', 'exports']);
    expect(missing[0]?.message).toContain('export a function render');
    const thrown = problems(
      await checkTemplate(moduleWith('', "throw new Error('boom');\n"), OPTIONS),
    );
    expect(thrown[0]).toMatchObject({ stage: 'exports', line: 3 });
    expect(thrown[0]?.message).toContain('boom');
  });

  it('names the line of syntax errors and imports', async () => {
    const syntax = problems(await checkTemplate(moduleWith('ctx.fillRect(0, 0,, 1);'), OPTIONS));
    expect(syntax[0]).toMatchObject({ stage: 'compile', line: 4 });
    const imported = problems(
      await checkTemplate("import x from 'y';\n" + moduleWith(''), OPTIONS),
    );
    expect(imported[0]).toMatchObject({ stage: 'compile', line: 1 });
  });

  it.each([
    ['lower third', lowerThird],
    ['animated title', animatedTitle],
    ['progress bar', progressBar],
    ['call to action', callToAction],
  ])('passes the instructions’ %s example', async (_name, source) => {
    const outcome = await checkTemplate(source, OPTIONS);
    expect(outcome.kind === 'failed' ? outcome.problems : []).toEqual([]);
    expect(outcome.kind).toBe('passed');
  });
});
