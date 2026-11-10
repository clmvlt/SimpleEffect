import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

import type { Plugin } from 'vite';

import { packagesOfModules } from './bundled-packages';

/**
 * Vite plugin listing the npm packages bundled into each output (prompt 30): it writes
 * `<noticesDir>/bundled/<label>-<entry>.json` (`BundledPackage[]`), merged with the Cargo crates
 * and the other components by `scripts/generate-notices.mjs`. Used by the renderer, its Workers
 * and the main and preload bundles.
 */
export function bundledPackagesPlugin(noticesDir: string, label: string): Plugin {
  return {
    name: 'simple-effect:bundled-packages',
    apply: 'build',
    generateBundle(_options, bundle) {
      const moduleIds: string[] = [];
      let entry = 'bundle';
      for (const output of Object.values(bundle)) {
        if (output.type !== 'chunk') continue;
        moduleIds.push(...output.moduleIds);
        if (output.isEntry) entry = output.name;
      }
      const folder = join(noticesDir, 'bundled');
      mkdirSync(folder, { recursive: true });
      writeFileSync(
        join(folder, `${label}-${entry}.json`),
        `${JSON.stringify(packagesOfModules(moduleIds), null, 2)}\n`,
      );
    },
  };
}
