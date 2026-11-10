import { join } from 'node:path';

import { describe, expect, it } from 'vitest';

import { resolveCoreBinaryPath } from './core-binary-path';
import { resolveLicencePaths, type ResourceContext } from './resource-paths';
import { parseThirdPartyNotices } from './third-party-notices';

/** The layout electron-builder gives each platform (`electron-builder.yml`). */
const installs = [
  {
    platform: 'win32',
    resourcesPath: join(
      'C:',
      'Users',
      'Marie',
      'AppData',
      'Local',
      'Programs',
      'SimpleEffect',
      'resources',
    ),
    core: 'se-core.exe',
  },
  {
    platform: 'darwin',
    resourcesPath: join('Applications', 'SimpleEffect.app', 'Contents', 'Resources'),
    core: 'se-core',
  },
  { platform: 'linux', resourcesPath: join('tmp', '.mount_SimpleE', 'resources'), core: 'se-core' },
] as const;

describe('packaged resource paths', () => {
  it.each(installs)('finds the core and the licences of a $platform install', (install) => {
    const context: ResourceContext = {
      isPackaged: true,
      resourcesPath: install.resourcesPath,
      appPath: join(install.resourcesPath, 'app.asar'),
    };
    expect(
      resolveCoreBinaryPath({ ...context, platform: install.platform, override: 'ignored' }),
    ).toBe(join(install.resourcesPath, 'bin', install.core));
    expect(resolveLicencePaths(context)).toEqual({
      folder: join(install.resourcesPath, 'LICENSES'),
      notices: join(install.resourcesPath, 'LICENSES', 'third-party-notices.json'),
    });
  });

  it('reads the repository’s files during development', () => {
    const appPath = join('repo', 'apps', 'desktop');
    expect(resolveLicencePaths({ isPackaged: false, resourcesPath: 'unused', appPath })).toEqual({
      folder: join('repo', 'LICENSES'),
      notices: join(appPath, 'out', 'notices', 'third-party-notices.json'),
    });
  });
});

describe('parseThirdPartyNotices', () => {
  it('reads the generated document', () => {
    const notices = parseThirdPartyNotices(
      JSON.stringify({
        formatVersion: 1,
        entries: [
          { source: 'npm', name: 'react', version: '19.0.0', license: 'MIT', textIds: [1] },
          {
            source: 'component',
            name: 'FFmpeg',
            version: '9.0.2',
            license: 'GPL-3.0-or-later',
            url: 'https://ffmpeg.org',
            textIds: [],
          },
        ],
        texts: { 1: 'MIT License' },
      }),
    );
    expect(notices.entries.map((entry) => entry.name)).toEqual(['react', 'FFmpeg']);
    expect(notices.entries[1]?.url).toBe('https://ffmpeg.org');
    expect(notices.texts).toEqual({ 1: 'MIT License' });
  });

  it('refuses another format or a malformed entry', () => {
    expect(() => parseThirdPartyNotices('{"formatVersion":2,"entries":[],"texts":{}}')).toThrow(
      /format 1/,
    );
    expect(() =>
      parseThirdPartyNotices(
        '{"formatVersion":1,"entries":[{"source":"pip","name":"x","version":"1","license":"MIT","textIds":[]}],"texts":{}}',
      ),
    ).toThrow(/notice 0 is malformed/);
  });
});
