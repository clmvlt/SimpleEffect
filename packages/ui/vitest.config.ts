import { defineProject } from 'vitest/config';

export default defineProject({
  test: {
    name: 'ui',
    environment: 'jsdom',
    include: ['src/**/*.test.{ts,tsx}'],
    setupFiles: ['./vitest.setup.ts'],
    // Vitest blanks CSS modules by default; token tests read the token stylesheets with `?raw`.
    css: { include: [/src[/\\]tokens[/\\].+\.css/] },
  },
});
