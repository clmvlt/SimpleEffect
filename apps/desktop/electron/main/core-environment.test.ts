import { describe, expect, it } from 'vitest';

import { coreEnvironment } from './core-environment';

describe('coreEnvironment', () => {
  it('adds the data folder without dropping or mutating the parent environment', () => {
    const parent = { PATH: '/bin', SE_LOG: 'debug' };
    expect(coreEnvironment('/data/SimpleEffect', parent)).toEqual({
      PATH: '/bin',
      SE_LOG: 'debug',
      SE_DATA_DIR: '/data/SimpleEffect',
    });
    expect(parent).toEqual({ PATH: '/bin', SE_LOG: 'debug' });
  });

  it('overrides an inherited SE_DATA_DIR', () => {
    expect(coreEnvironment('/new', { SE_DATA_DIR: '/old' })['SE_DATA_DIR']).toBe('/new');
  });
});
