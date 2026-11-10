import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import babel from '@rolldown/plugin-babel';
import tailwindcss from '@tailwindcss/vite';
import react, { reactCompilerPreset } from '@vitejs/plugin-react';
import { defineConfig } from 'electron-vite';

import { bundledPackagesPlugin } from './tooling/bundled-packages-plugin';
import { contentSecurityPolicyPlugin } from './tooling/csp-plugin';

const projectRoot = import.meta.dirname;

/** Where the licence notices are assembled (`scripts/generate-notices.mjs`, prompt 30). */
const noticesDir = resolve(projectRoot, 'out/notices');

/** The `version` of a `package.json`. */
function packageVersion(path: string): string {
  return (JSON.parse(readFileSync(path, 'utf8')) as { version: string }).version;
}

/** The app version, shown in the status bar (`__APP_VERSION__`, declared in `src/env.d.ts`). */
const appVersion = packageVersion(resolve(projectRoot, 'package.json'));

/** The engine's version, shown in « À propos » (`__ENGINE_VERSION__`). */
const engineVersion = packageVersion(resolve(projectRoot, '../../packages/engine/package.json'));

/**
 * Three bundles: Electron main (ESM), preload (CommonJS, required by `sandbox: true`) and the
 * React renderer. All runtime dependencies are bundled, so the packaged app ships no node_modules.
 */
export default defineConfig({
  main: {
    plugins: [bundledPackagesPlugin(noticesDir, 'main')],
    build: {
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'electron/main/index.ts') },
      },
    },
  },
  preload: {
    plugins: [bundledPackagesPlugin(noticesDir, 'preload')],
    build: {
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'electron/preload/index.ts') },
        output: { format: 'cjs', entryFileNames: '[name].cjs' },
      },
    },
  },
  renderer: {
    root: resolve(projectRoot, 'src'),
    define: {
      __APP_VERSION__: JSON.stringify(appVersion),
      __ENGINE_VERSION__: JSON.stringify(engineVersion),
    },
    plugins: [
      contentSecurityPolicyPlugin(),
      react(),
      babel({ presets: [reactCompilerPreset()] }),
      tailwindcss(),
      bundledPackagesPlugin(noticesDir, 'renderer'),
    ],
    // The export Worker (`@se/engine/export`) is a module Worker that imports lazily: the default
    // `iife` format cannot split its chunks.
    worker: { format: 'es', plugins: () => [bundledPackagesPlugin(noticesDir, 'worker')] },
    build: {
      outDir: resolve(projectRoot, 'out/renderer'),
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'src/index.html') },
      },
    },
  },
});
