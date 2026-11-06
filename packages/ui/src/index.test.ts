import { describe, expect, it } from 'vitest';

describe('package entry point', () => {
  it('loads without side-effect errors', async () => {
    await expect(import('./index')).resolves.toBeDefined();
  });
});
