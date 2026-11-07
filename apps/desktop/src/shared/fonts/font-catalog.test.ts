import type { Command, CoreEndpoint, FontFamily } from '@se/protocol';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { catalogFaces, fontCatalogStore, loadFontCatalog, resetFontCatalog } from '.';

const inter: FontFamily = {
  name: 'Inter',
  source: 'bundled',
  faces: [
    {
      id: 'bundled-inter-latin-wght-normal',
      weightMin: 100,
      weightMax: 900,
      italic: false,
      unicodeRange: 'U+0000-00FF',
      format: 'woff2',
    },
  ],
};

afterEach(resetFontCatalog);

describe('the font catalogue', () => {
  it('is listed once and kept', async () => {
    const send = vi.fn((command: Command) =>
      Promise.resolve({ kind: command.kind, families: [inter] } as never),
    );
    await Promise.all([loadFontCatalog({ send }), loadFontCatalog({ send })]);
    await loadFontCatalog({ send });
    expect(send).toHaveBeenCalledTimes(1);
    expect(fontCatalogStore.getState().families).toEqual([inter]);
  });

  it('gives the engine each face with its tokenised URL', () => {
    const endpoint: CoreEndpoint = { url: 'http://127.0.0.1:4000', token: 'secret' };
    expect(catalogFaces([inter], endpoint)).toEqual([
      {
        family: 'Inter',
        url: 'http://127.0.0.1:4000/media/fonts/bundled-inter-latin-wght-normal?t=secret',
        weightMin: 100,
        weightMax: 900,
        italic: false,
        unicodeRange: 'U+0000-00FF',
        format: 'woff2',
      },
    ]);
  });
});
