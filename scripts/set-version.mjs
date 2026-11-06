// `node scripts/set-version.mjs <version>`: writes the app version into every package.json of the
// workspace and into [workspace.package] of Cargo.toml. Used by the release workflow with the tag
// version (`v1.2.3-rc.1` → `1.2.3-rc.1`). Cargo.lock is refreshed by the next cargo build.
import { existsSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import { repoRoot } from './lib/run-command.mjs';

const SEMVER = /^\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;
const WORKSPACE_GROUPS = ['apps', 'packages'];

const version = process.argv[2];
if (version === undefined || !SEMVER.test(version)) {
  console.error('Usage: node scripts/set-version.mjs <semver>   e.g. 1.2.3 or 1.2.3-rc.1');
  process.exit(1);
}

/** @returns {string[]} */
function packageJsonPaths() {
  const paths = [join(repoRoot, 'package.json')];
  for (const group of WORKSPACE_GROUPS) {
    for (const entry of readdirSync(join(repoRoot, group), { withFileTypes: true })) {
      const manifest = join(repoRoot, group, entry.name, 'package.json');
      if (entry.isDirectory() && existsSync(manifest)) {
        paths.push(manifest);
      }
    }
  }
  return paths;
}

for (const path of packageJsonPaths()) {
  const manifest = JSON.parse(readFileSync(path, 'utf8'));
  manifest.version = version;
  writeFileSync(path, `${JSON.stringify(manifest, null, 2)}\n`);
}

const cargoPath = join(repoRoot, 'Cargo.toml');
const cargo = readFileSync(cargoPath, 'utf8');
const updated = cargo.replace(
  /(\[workspace\.package\][^[]*?\nversion\s*=\s*)"[^"]*"/,
  (_match, prefix) => `${prefix}"${version}"`,
);
if (updated === cargo && !cargo.includes(`version = "${version}"`)) {
  console.error('Cannot find `version = "…"` under [workspace.package] in Cargo.toml.');
  process.exit(1);
}
writeFileSync(cargoPath, updated);

console.log(
  `Version set to ${version} in Cargo.toml and ${packageJsonPaths().length} package.json files.`,
);
