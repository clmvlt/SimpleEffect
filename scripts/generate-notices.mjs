// `node scripts/generate-notices.mjs` (run by `pnpm build` after the desktop bundles, prompt 30):
// writes apps/desktop/out/notices/third-party-notices.json and THIRD-PARTY-NOTICES.txt from
// LICENSES/components.json, the npm packages the Vite plugin found in each bundle
// (apps/desktop/out/notices/bundled/*.json) and the Cargo crates linked into se-core.
import { execFileSync } from 'node:child_process';
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';

import { repoRoot } from './lib/run-command.mjs';
import {
  licenceFilesIn,
  noticesDocument,
  noticesText,
  npmLicenceOf,
  npmUrlOf,
  shippedCrates,
} from './lib/notices.mjs';

const noticesDir = resolve(repoRoot, 'apps/desktop/out/notices');
const bundledDir = join(noticesDir, 'bundled');

/** @typedef {import('./lib/notices.mjs').NoticeEntry} NoticeEntry */

/** @param {string} directory @returns {NoticeEntry['texts']} */
function licenceTexts(directory) {
  if (!existsSync(directory)) return [];
  return licenceFilesIn(readdirSync(directory)).map((file) => ({
    file,
    text: readFileSync(join(directory, file), 'utf8'),
  }));
}

/** @returns {NoticeEntry[]} */
function componentEntries() {
  const { components } = JSON.parse(
    readFileSync(join(repoRoot, 'LICENSES/components.json'), 'utf8'),
  );
  return components.map((/** @type {Record<string, any>} */ component) => {
    const texts = component.files.map((/** @type {string} */ file) => {
      const path = join(repoRoot, file);
      if (!existsSync(path)) throw new Error(`${file} (LICENSES/components.json) does not exist`);
      return { file, text: readFileSync(path, 'utf8') };
    });
    return {
      source: 'component',
      name: component.name,
      version: component.version,
      license: component.license,
      url: component.url,
      ...(component.note === undefined ? {} : { note: component.note }),
      texts,
    };
  });
}

/** @returns {NoticeEntry[]} */
function npmEntries() {
  if (!existsSync(bundledDir)) {
    throw new Error(`${bundledDir} is missing: build the desktop app first (pnpm build)`);
  }
  /** @type {Map<string, string>} */
  const directories = new Map();
  for (const file of readdirSync(bundledDir).filter((name) => name.endsWith('.json'))) {
    for (const found of JSON.parse(readFileSync(join(bundledDir, file), 'utf8'))) {
      directories.set(found.directory, found.name);
    }
  }
  /** @type {Map<string, NoticeEntry>} */
  const byPackage = new Map();
  for (const directory of directories.keys()) {
    const manifest = JSON.parse(readFileSync(join(directory, 'package.json'), 'utf8'));
    const url = npmUrlOf(manifest);
    byPackage.set(`${manifest.name}@${manifest.version}`, {
      source: 'npm',
      name: manifest.name,
      version: manifest.version,
      license: npmLicenceOf(manifest),
      ...(url === undefined ? {} : { url }),
      texts: licenceTexts(directory),
    });
  }
  return [...byPackage.values()];
}

/** @returns {NoticeEntry[]} */
function cargoEntries() {
  const host = /^host: (.+)$/m.exec(execFileSync('rustc', ['-vV'], { encoding: 'utf8' }))?.[1];
  if (host === undefined) throw new Error('rustc -vV did not print the host triple');
  const metadata = JSON.parse(
    execFileSync(
      'cargo',
      ['metadata', '--format-version', '1', '--locked', '--filter-platform', host.trim()],
      { cwd: repoRoot, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 },
    ),
  );
  return shippedCrates(metadata, 'se-core').map((crate) => {
    const url = crate.repository ?? crate.homepage ?? undefined;
    return {
      source: 'cargo',
      name: crate.name,
      version: crate.version,
      license: crate.license ?? (crate.license_file == null ? 'UNKNOWN' : 'see file'),
      ...(url === undefined || url === null ? {} : { url }),
      texts: licenceTexts(dirname(crate.manifest_path)),
    };
  });
}

const entries = [...componentEntries(), ...npmEntries(), ...cargoEntries()];
const document = noticesDocument(entries);
writeFileSync(join(noticesDir, 'third-party-notices.json'), JSON.stringify(document));
writeFileSync(join(noticesDir, 'THIRD-PARTY-NOTICES.txt'), noticesText(document));
const counts = { component: 0, npm: 0, cargo: 0 };
for (const entry of entries) counts[entry.source] += 1;
console.log(
  `Notices: ${counts.component} components, ${counts.npm} npm packages, ${counts.cargo} crates, ` +
    `${Object.keys(document.texts).length} distinct licence texts → ${noticesDir}`,
);
