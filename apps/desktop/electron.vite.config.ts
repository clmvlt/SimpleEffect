import { resolve } from 'node:path';

import babel from '@rolldown/plugin-babel';
import tailwindcss from '@tailwindcss/vite';
import react, { reactCompilerPreset } from '@vitejs/plugin-react';
import { defineConfig } from 'electron-vite';

import { contentSecurityPolicyPlugin } from './tooling/csp-plugin';

const projectRoot = import.meta.dirname;

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
    plugins: [
      contentSecurityPolicyPlugin(),
      react(),
      babel({ presets: [reactCompilerPreset()] }),
      tailwindcss(),
    ],
    build: {
      outDir: resolve(projectRoot, 'out/renderer'),
      rolldownOptions: {
        input: { index: resolve(projectRoot, 'src/index.html') },
      },
    },
  },
});
