import { describe, expect, it } from 'vitest';

import { byteSize } from './byte-size';

describe('byteSize', () => {
  it('uses binary multiples with French units and decimals', () => {
    expect(byteSize(512, 'fr')).toBe('512 o');
    expect(byteSize(1536, 'fr')).toBe('1,5 Ko');
    expect(byteSize(13_002_342, 'fr')).toBe('12,4 Mo');
    expect(byteSize(0, 'fr')).toBe('0 o');
  });
});
