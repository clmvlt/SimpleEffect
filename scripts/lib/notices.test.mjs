import { describe, expect, it } from 'vitest';

import {
  licenceFilesIn,
  noticesDocument,
  noticesText,
  npmLicenceOf,
  npmUrlOf,
  shippedCrates,
} from './notices.mjs';

/** @param {string} name @param {string[]} [kinds] */
function crate(name, kinds = ['lib']) {
  return {
    id: `${name}-id`,
    name,
    version: '1.0.0',
    license: 'MIT',
    manifest_path: `/c/${name}/Cargo.toml`,
    targets: [{ kind: kinds }],
  };
}

/** @param {string} pkg @param {string | null} kind */
function dep(pkg, kind = null) {
  return { pkg: `${pkg}-id`, dep_kinds: [{ kind }] };
}

describe('notices', () => {
  it('lists the crates linked into the binary only', () => {
    const metadata = {
      packages: [
        crate('se-core', ['bin']),
        crate('se-app'),
        crate('serde'),
        crate('serde_derive', ['proc-macro']),
        crate('syn'),
        crate('cc'),
        crate('proptest'),
        crate('zip'),
      ],
      workspace_members: ['se-core-id', 'se-app-id'],
      resolve: {
        nodes: [
          {
            id: 'se-core-id',
            deps: [dep('se-app'), dep('cc', 'build'), dep('proptest', 'dev')],
          },
          { id: 'se-app-id', deps: [dep('serde'), dep('zip')] },
          { id: 'serde-id', deps: [dep('serde_derive')] },
          { id: 'serde_derive-id', deps: [dep('syn')] },
        ],
      },
    };
    expect(shippedCrates(metadata, 'se-core').map((found) => found.name)).toEqual(['serde', 'zip']);
    expect(() => shippedCrates(metadata, 'serde')).toThrow(/no workspace crate serde/);
  });

  it('finds licence files and npm licence forms', () => {
    expect(
      licenceFilesIn(['index.js', 'LICENSE-MIT', 'license.md', 'NOTICE', 'COPYING', 'licensed.js']),
    ).toEqual(['COPYING', 'LICENSE-MIT', 'NOTICE', 'license.md']);
    expect(npmLicenceOf({ license: 'MIT' })).toBe('MIT');
    expect(npmLicenceOf({ license: { type: 'BSD-3-Clause' } })).toBe('BSD-3-Clause');
    expect(npmLicenceOf({ licenses: [{ type: 'MIT' }, { type: 'Apache-2.0' }] })).toBe(
      'MIT OR Apache-2.0',
    );
    expect(npmLicenceOf({})).toBe('UNKNOWN');
    expect(npmUrlOf({ repository: { url: 'git+https://github.com/a/b.git' } })).toBe(
      'https://github.com/a/b',
    );
  });

  it('stores identical texts once and renders every entry', () => {
    const mit = 'MIT License\r\n\r\nCopyright (c) Someone';
    const document = noticesDocument([
      {
        source: 'cargo',
        name: 'zip',
        version: '2.0.0',
        license: 'MIT',
        texts: [{ file: 'LICENSE', text: mit }],
      },
      {
        source: 'npm',
        name: 'react',
        version: '19.0.0',
        license: 'MIT',
        texts: [{ file: 'LICENSE', text: `\u{FEFF}${mit}\n` }],
      },
      {
        source: 'component',
        name: 'FFmpeg',
        version: '9.0.2',
        license: 'GPL-3.0-or-later',
        texts: [],
      },
    ]);
    expect(document.entries.map((entry) => entry.name)).toEqual(['FFmpeg', 'react', 'zip']);
    expect(document.entries.map((entry) => entry.textIds)).toEqual([[], [1], [1]]);
    expect(document.texts).toEqual({ 1: 'MIT License\n\nCopyright (c) Someone' });
    const text = noticesText(document);
    expect(text).toContain('- [npm] react 19.0.0 — MIT\n  Texts: #1');
    expect(text).toContain('Text #1');
  });
});
