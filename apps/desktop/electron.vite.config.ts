import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import babel from '@rolldown/plugin-babel';
import tailwindcss from '@tailwindcss/vite';
import react, { reactCompilerPreset } from '@vitejs/plugin-react';
import { defineConfig } from 'electron-vite';

import { contentSecurityPolicyPlugin } from './tooling/csp-plugin';

const projectRoot = import.meta.dirname;

/** The app version, shown in the status bar (`__APP_VERSION__`, declared in `src/env.d.ts`). */
const appVersion = (
  JSON.parse(readFileSync(resolve(projectRoot, 'package.json'), 'utf8')) as { version: string }
).version;

/**
 * Three bundles: Electron main (ESM), preload (CommonJS, required by `sandbox: true`) and the
 * React renderer. All runtime dependencies are bundled, so the packaged app ships no node_modules.
 */
export default defineConfig({
  main: {
    build: {
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'electron/main/index.ts') },
      },
    },
  },
  preload: {
    build: {
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'electron/preload/index.ts') },
        output: { format: 'cjs', entryFileNames: '[name].cjs' },
      },
    },
  },
  renderer: {
    root: resolve(projectRoot, 'src'),
    define: { __APP_VERSION__: JSON.stringify(appVersion) },
    plugins: [
      contentSecurityPolicyPlugin(),
      react(),
      babel({ presets: [reactCompilerPreset()] }),
      tailwindcss(),
    ],
    // The export Worker (`@se/engine/export`) is a module Worker that imports lazily: the default
    // `iife` format cannot split its chunks.
    worker: { format: 'es' },
    build: {
      outDir: resolve(projectRoot, 'out/renderer'),
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'src/index.html') },
      },
    },
  },
});
