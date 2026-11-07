import { defineConfig } from 'vitest/config';

/**
 * Root Vitest entry point: each workspace package (and `scripts/`) owns its own Vitest config
 * and is run as a separate project, so `pnpm test` covers the whole monorepo. The engine's
 * golden-frame tests are a second project of that package, run in headless Chromium.
 */
export default defineConfig({
  test: {
    projects: ['packages/*', 'apps/*', 'scripts', 'packages/engine/vitest.browser.config.ts'],
  },
});
