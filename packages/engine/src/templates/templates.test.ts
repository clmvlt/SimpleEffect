import { describe, expect, it } from 'vitest';

import { checkDefaults, checkInstants } from './check-plan';
import { compileTemplate } from './compile';
import { templateWorkerSource } from './sandbox-source';
import { createHelpers } from './sandbox/helpers';

describe('compileTemplate', () => {
  it('removes types and keeps every line in place', () => {
    const result = compileTemplate(
      [
        "export const meta = { name: 'x', defaultDuration: 1 };",
        'export const params = {};',
        'export function render(ctx: any, time: number): void {',
        '  const size: number = time * 10;',
        '  ctx.fillRect(0, 0, size, size);',
        '}',
      ].join('\n'),
    );
    expect(result.ok).toBe(true);
    if (!result.ok) return;
    const lines = result.compiled.code.split('\n');
    expect(lines[3]).toContain('const size = time * 10;');
    expect(lines[4]).toContain('ctx.fillRect(0, 0, size, size);');
    expect(result.compiled.code).toContain('exports.render = render');
  });

  it('names the line and column of syntax errors', () => {
    const result = compileTemplate('export function render(ctx) {\n  ctx.fillRect(0,, 1);\n}');
    expect(result.ok).toBe(false);
    if (result.ok) return;
    expect(result.problems[0]).toMatchObject({ line: 2 });
    expect(result.problems[0]?.message).toMatch(/^syntax error: /);
    expect(result.problems[0]?.column).toBeGreaterThan(0);
  });

  it('refuses imports and requires, not words starting with import', () => {
    const imported = compileTemplate("import x from 'y';\nexport const a = 1;");
    expect(imported.ok ? [] : imported.problems.map((problem) => problem.line)).toEqual([1]);
    const required = compileTemplate("const fs = require('fs');");
    expect(required.ok).toBe(false);
    expect(compileTemplate('const importance = 1; importScripts;').ok).toBe(true);
  });
});

describe('templateWorkerSource', () => {
  it('puts the module after a counted prefix declaring nothing at the top level', () => {
    const { source, lineOffset } = templateWorkerSource({ code: 'MODULE_LINE_ONE\nsecond' });
    const lines = source.split('\n');
    expect(lines[lineOffset]).toBe('MODULE_LINE_ONE');
    expect(lines[0]).toBe('"use strict";');
    expect(source.trimEnd().endsWith('});')).toBe(true);
    // Everything before the module is one call: the runtime as an expression, then its arguments.
    expect(lines[1]).toMatch(/^\(function workerMain\(/);
    expect(lines[lineOffset - 2]).toMatch(/^function \(exports, globalThis, self/);
  });
});

describe('check plan', () => {
  it('reads the declared length and defaults leniently', () => {
    const defaults = checkDefaults(
      '{"name":"x","defaultDuration":3}',
      '[["title",{"type":"string","default":"Hi"}],["font",{"type":"string","format":"font","default":"Bebas Neue"}],["logo",{"type":"string","format":"asset"}],["size",{"type":"number"}]]',
    );
    expect(defaults).toEqual({
      duration: 3,
      params: { title: 'Hi', font: 'Bebas Neue', logo: null, size: null },
      families: ['Bebas Neue'],
    });
    expect(checkDefaults('nope', 'nope')).toEqual({ duration: 5, params: {}, families: [] });
  });

  it('renders the edges, the entrance, the middle and the exit', () => {
    expect(checkInstants(4)).toEqual({ timed: [0, 0.4, 2, 3.6, 3.983], pictures: [0.4, 2, 3.6] });
    expect(checkInstants(1).pictures).toEqual([0.2, 0.5, 0.8]);
  });
});

describe('template helpers', () => {
  const helpers = createHelpers();

  it('eases within [0, 1] and clamps outside', () => {
    expect(helpers.ease['outCubic']?.(0)).toBe(0);
    expect(helpers.ease['outCubic']?.(1)).toBe(1);
    expect(helpers.ease['outCubic']?.(2)).toBe(1);
    expect(helpers.ease['inOutQuad']?.(0.5)).toBeCloseTo(0.5);
    expect(helpers.ease['outBack']?.(0.6)).toBeGreaterThan(1);
  });

  it('gives progress, lerp and clamp', () => {
    expect(helpers.progress(1.5, 1, 2)).toBe(0.5);
    expect(helpers.progress(0, 1, 2)).toBe(0);
    expect(helpers.progress(3, 2, 2)).toBe(1);
    expect(helpers.lerp(10, 20, 0.25)).toBe(12.5);
    expect(helpers.clamp(5, 0, 1)).toBe(1);
  });

  it('draws the same random numbers from the same seed', () => {
    const first = helpers.random('stars');
    const second = helpers.random('stars');
    const values = [first(), first(), first()];
    expect([second(), second(), second()]).toEqual(values);
    expect(values.every((value) => value >= 0 && value < 1)).toBe(true);
    expect(helpers.random('other')()).not.toBe(values[0]);
  });

  it('mixes colours and sets their alpha', () => {
    expect(helpers.mixColor('#000000', '#ffffff', 0.5)).toBe('#808080ff');
    expect(helpers.withAlpha('#ff0000', 0.5)).toBe('#ff000080');
    expect(() => helpers.mixColor('red', '#fff', 0)).toThrow('#rrggbb');
  });
});
