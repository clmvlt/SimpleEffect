import { describe, expect, it } from 'vitest';

import { parseChooseFolderOptions, parseRevealPath } from './ipc-arguments';

describe('parseChooseFolderOptions', () => {
  it('keeps valid options', () => {
    expect(parseChooseFolderOptions({ title: 'T', buttonLabel: 'B' })).toEqual({
      title: 'T',
      buttonLabel: 'B',
    });
    expect(
      parseChooseFolderOptions({ title: 'T', buttonLabel: 'B', defaultPath: '/home' }),
    ).toEqual({ title: 'T', buttonLabel: 'B', defaultPath: '/home' });
  });

  it('refuses malformed options', () => {
    expect(() => parseChooseFolderOptions(null)).toThrow(TypeError);
    expect(() => parseChooseFolderOptions({ title: 1, buttonLabel: 'B' })).toThrow(TypeError);
    expect(() =>
      parseChooseFolderOptions({ title: 'T', buttonLabel: 'B', defaultPath: 3 }),
    ).toThrow(TypeError);
  });
});

describe('parseRevealPath', () => {
  it('accepts absolute paths only', () => {
    expect(parseRevealPath('/projects/Film.seproj')).toBe('/projects/Film.seproj');
    expect(() => parseRevealPath('Film.seproj')).toThrow(TypeError);
    expect(() => parseRevealPath(42)).toThrow(TypeError);
  });
});
