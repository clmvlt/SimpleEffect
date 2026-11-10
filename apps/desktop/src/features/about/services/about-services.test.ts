import type { AiStatus } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import type { RuntimeInfo } from '../../../shared/desktop';
import { aboutTabOf } from '../store/about-dialog-store';
import { versionRows, versionsText } from './about-versions';
import { collectClientDiagnostics, diagnosticsFileName } from './client-diagnostics';
import { filterNotices, listedNotices, sourceCounts } from './licence-list';

const runtime: RuntimeInfo = {
  electron: '45.2.3',
  chrome: '146.0.7680.80',
  node: '24.11.0',
  v8: '14.6.1',
  platform: 'win32',
  arch: 'x64',
  packaged: true,
};

const installedAi = {
  provider: {
    id: 'claudeCode',
    name: 'Claude Code',
    command: 'claude',
    loginCommand: '/login',
    helpUrl: 'https://example.test/help',
  },
  installation: { kind: 'installed', path: 'C:/bin/claude.exe', version: '2.1.317' },
  auth: { kind: 'loggedIn', accountKind: null },
  capabilities: { models: [], efforts: [], sessions: true },
} as unknown as AiStatus;

describe('about services', () => {
  it('lists every version and copies them as text', () => {
    const rows = versionRows({
      app: '0.1.0',
      engine: '0.1.0',
      core: '0.1.0',
      runtime,
      ai: installedAi,
    });
    expect(rows.map((row) => [row.id, row.version])).toEqual([
      ['app', '0.1.0'],
      ['core', '0.1.0'],
      ['engine', '0.1.0'],
      ['electron', '45.2.3'],
      ['chromium', '146.0.7680.80'],
      ['ai', '2.1.317'],
    ]);
    const unknown = versionRows({
      app: '0.1.0',
      engine: '0.1.0',
      core: null,
      runtime: null,
      ai: null,
    });
    expect(versionsText(unknown.slice(1, 2), (row) => row.id)).toBe('core: —');
  });

  it('collects the renderer’s diagnostics without failing on a broken probe', async () => {
    const diagnostics = await collectClientDiagnostics(
      { app: '0.1.0', engine: '0.1.0', runtime },
      {
        gpu: () => 'ANGLE (NVIDIA GeForce RTX 3080)',
        codecSupport: ({ name }) =>
          name === 'AV1'
            ? Promise.reject(new Error('no'))
            : Promise.resolve({ decode: true, encode: name === 'H.264' }),
        locale: 'fr-FR',
      },
    );
    expect(diagnostics.components.map((component) => component.name)).toEqual([
      '@se/engine',
      'Electron',
      'Chromium',
      'Node.js',
      'V8',
      'Platform',
    ]);
    expect(diagnostics.codecs).toEqual([
      { codec: 'H.264', decode: true, encode: true },
      { codec: 'HEVC', decode: true, encode: false },
      { codec: 'VP9', decode: true, encode: false },
      { codec: 'AV1', decode: false, encode: false },
    ]);
    expect(diagnostics.gpu).toContain('RTX 3080');
    expect(diagnosticsFileName(new Date(2026, 10, 10, 18, 5))).toBe(
      'SimpleEffect-diagnostic-2026-11-10-1805.zip',
    );
  });

  it('searches the licences by name and licence', () => {
    const notices = listedNotices([
      {
        source: 'component',
        name: 'FFmpeg',
        version: '9.0.2',
        license: 'GPL-3.0-or-later',
        textIds: [1],
      },
      { source: 'npm', name: 'react', version: '19.0.0', license: 'MIT', textIds: [2] },
      { source: 'cargo', name: 'zip', version: '2.0.0', license: 'MIT', textIds: [2] },
    ]);
    expect(filterNotices(notices, 'mit').map((notice) => notice.key)).toEqual([
      'npm:react@19.0.0',
      'cargo:zip@2.0.0',
    ]);
    expect(filterNotices(notices, ' FFMPEG gpl ')).toHaveLength(1);
    expect(filterNotices(notices, '')).toHaveLength(3);
    expect(sourceCounts(notices)).toEqual({ component: 1, npm: 1, cargo: 1 });
    expect(aboutTabOf('licences')).toBe('licences');
    expect(aboutTabOf('other')).toBe('application');
  });
});
