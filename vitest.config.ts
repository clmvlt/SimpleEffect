import { defineConfig } from 'vitest/config';

/**
 * Root Vitest entry point: each workspace package owns its own `vitest.config.ts`
 * and is run as a separate project, so `pnpm test` covers the whole monorepo.
 */
export default defineConfig({
  test: {
    projects: ['packages/*', 'apps/*'],
  },
});
