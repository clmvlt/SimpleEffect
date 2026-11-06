import { defineProject } from 'vitest/config';

/**
 * Electron and tooling tests run in Node; renderer tests that render React (`*.test.tsx`) opt
 * into jsdom with a `@vitest-environment jsdom` docblock. `vitest.setup.ts` only touches the DOM
 * when one exists.
 */
export default defineProject({
  define: { __APP_VERSION__: JSON.stringify('0.0.0-test') },
  test: {
    name: 'desktop',
    environment: 'node',
    include: ['electron/**/*.test.ts', 'tooling/**/*.test.ts', 'src/**/*.test.{ts,tsx}'],
    setupFiles: ['./vitest.setup.ts'],
  },
});
