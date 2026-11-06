import { spawnSync } from 'node:child_process';
import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { findRawColors, scanForRawColors } from './raw-colors.mjs';

const CHECK_SCRIPT = join(dirname(fileURLToPath(import.meta.url)), '..', 'check-raw-colors.mjs');

describe('findRawColors', () => {
  it('finds hex colours and colour functions with their position', () => {
    const source = "const a = 'bg-[#1e1e1e]';\nborder: 1px solid rgb(0 0 0 / 50%);\n";
    expect(findRawColors(source)).toEqual([
      { line: 1, column: 16, literal: '#1e1e1e' },
      { line: 2, column: 19, literal: 'rgb(' },
    ]);
  });

  it.each(['#fff', '#FFFF', '#a1b2c3', '#a1b2c3d4', 'rgba(', 'hsl(', 'oklch(', 'color(display-p3'])(
    'flags %s',
    (literal) => {
      expect(findRawColors(`x ${literal} 1)`)).toHaveLength(1);
    },
  );

  it.each([
    'bg-panel text-text-muted',
    'var(--color-accent)',
    "href='#header'",
    'animation: fade-in',
    'getColor(item)',
    '#root',
  ])('accepts %s', (source) => {
    expect(findRawColors(source)).toEqual([]);
  });
});

describe('check-raw-colors script', () => {
  /** @type {string} */
  let root;

  beforeEach(async () => {
    root = await mkdtemp(join(tmpdir(), 'raw-colors-'));
  });

  afterEach(async () => {
    await rm(root, { recursive: true, force: true });
  });

  /** @param {string} path @param {string} content */
  async function writeFixture(path, content) {
    await mkdir(dirname(join(root, path)), { recursive: true });
    await writeFile(join(root, path), content);
  }

  function runCheck() {
    return spawnSync(process.execPath, [CHECK_SCRIPT, root], { encoding: 'utf8' });
  }

  it('allows colours inside tokens folders', async () => {
    await writeFixture('packages/ui/src/tokens/colors.css', '--color-bg: oklch(0.2 0 0);');
    await writeFixture('apps/desktop/src/app/App.tsx', "<main className='bg-bg' />");
    expect(await scanForRawColors(root)).toEqual([]);
    expect(runCheck().status).toBe(0);
  });

  it('fails on a raw colour in a component and names the file', async () => {
    await writeFixture(
      'packages/ui/src/primitives/bad/bad.tsx',
      "<div className='text-[#ff0000]' />",
    );
    const result = runCheck();
    expect(result.status).toBe(1);
    expect(result.stderr).toContain('packages/ui/src/primitives/bad/bad.tsx:1:23');
  });

  it('fails on a raw colour in the desktop stylesheet', async () => {
    await writeFixture('apps/desktop/src/app/styles.css', 'body { color: rgb(255 0 0); }');
    expect(runCheck().status).toBe(1);
  });
});
