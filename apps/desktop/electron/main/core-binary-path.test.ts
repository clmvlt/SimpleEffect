import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { resolveCoreBinaryPath, type CoreBinaryContext } from './core-binary-path';

const base: CoreBinaryContext = {
  isPackaged: false,
  resourcesPath: join('opt', 'SimpleEffect', 'resources'),
  appPath: join('repo', 'apps', 'desktop'),
  platform: 'linux',
  override: undefined,
};

describe('resolveCoreBinaryPath', () => {
  it('uses the debug build of the workspace during development', () => {
    expect(resolveCoreBinaryPath(base)).toBe(join('repo', 'target', 'debug', 'se-core'));
  });

  it('adds the .exe extension on Windows', () => {
    expect(resolveCoreBinaryPath({ ...base, platform: 'win32' })).toBe(
      join('repo', 'target', 'debug', 'se-core.exe'),
    );
  });

  it('uses the bundled binary in a packaged app', () => {
    expect(resolveCoreBinaryPath({ ...base, isPackaged: true })).toBe(
      join('opt', 'SimpleEffect', 'resources', 'bin', 'se-core'),
    );
  });

  it('honours the override during development only', () => {
    const override = join('custom', 'se-core');
    expect(resolveCoreBinaryPath({ ...base, override })).toBe(override);
    expect(resolveCoreBinaryPath({ ...base, override, isPackaged: true })).toBe(
      join('opt', 'SimpleEffect', 'resources', 'bin', 'se-core'),
    );
  });
});
