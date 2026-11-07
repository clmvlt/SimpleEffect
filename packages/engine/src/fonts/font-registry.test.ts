import { describe, expect, it } from 'vitest';

import { FontRegistry, type CatalogFace, type FontFaceLike } from './font-registry';

class FakeFace implements FontFaceLike {
  loads = 0;
  private resolve: () => void = () => undefined;
  readonly done = new Promise<void>((resolve) => {
    this.resolve = resolve;
  });

  constructor(
    readonly family: string,
    readonly source: string,
    readonly descriptors: FontFaceDescriptors,
  ) {}

  load(): Promise<unknown> {
    this.loads += 1;
    return this.done;
  }

  finish(): void {
    this.resolve();
  }
}

function setup() {
  const added = new Set<FakeFace>();
  const registry = new FontRegistry<FakeFace>(
    { add: (face) => added.add(face), delete: (face) => added.delete(face) },
    (family, source, descriptors) => new FakeFace(family, source, descriptors),
  );
  return { registry, added };
}

const face = (family: string, url: string, italic = false): CatalogFace => ({
  family,
  url,
  weightMin: 100,
  weightMax: 900,
  italic,
  unicodeRange: 'U+0000-00FF',
  format: 'woff2',
});

const request = (family: string) => ({ family, weight: 700, italic: false });

describe('the font registry', () => {
  it('registers catalogue faces with their weights, style and subset', () => {
    const { registry, added } = setup();
    registry.setCatalog([face('Inter', '/fonts/a'), face('Inter', '/fonts/b', true)]);
    const faces = [...added];
    expect(faces).toHaveLength(2);
    expect(faces[0]?.source).toBe('url("/fonts/a") format("woff2")');
    expect(faces[1]?.descriptors).toEqual({
      weight: '100 900',
      style: 'italic',
      unicodeRange: 'U+0000-00FF',
    });
    registry.setCatalog([]);
    expect(added.size).toBe(0);
  });

  it('loads every face of a family once, the first time a text needs it', async () => {
    const { registry, added } = setup();
    registry.setCatalog([face('Inter', '/a'), face('Inter', '/b', true), face('Oswald', '/c')]);
    const pending = registry.ensure([request('inter'), request('Inter')]);
    expect(pending).not.toBeNull();
    const [regular, italic, oswald] = [...added];
    expect([regular?.loads, italic?.loads, oswald?.loads]).toEqual([1, 1, 0]);
    regular?.finish();
    italic?.finish();
    await pending;
    expect(registry.ensure([request('Inter')])).toBeNull();
    expect(regular?.loads).toBe(1);
  });

  it('reports families the catalogue lacks, only once a catalogue exists', () => {
    const { registry } = setup();
    expect(registry.missing([request('Comic')])).toEqual([]);
    registry.setCatalog([face('Inter', '/a')]);
    expect(registry.missing([request('Comic'), request('inter'), request('Comic')])).toEqual([
      'Comic',
    ]);
    expect(registry.ensure([request('Comic')])).toBeNull();
  });
});
