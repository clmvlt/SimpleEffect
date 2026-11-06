import { defineProject } from 'vitest/config';

export default defineProject({
  test: {
    name: 'desktop',
    environment: 'node',
    include: ['electron/**/*.test.ts', 'tooling/**/*.test.ts', 'src/**/*.test.{ts,tsx}'],
  },
});
