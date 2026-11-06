import { defineProject } from 'vitest/config';

/** Tests of the repository scripts (`scripts/**`), run in Node. */
export default defineProject({
  test: {
    name: 'scripts',
    environment: 'node',
    include: ['**/*.test.mjs'],
  },
});
