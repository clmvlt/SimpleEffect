import type { Format, ItemEntity } from '@se/protocol';
import type { TextLayer } from '@se/engine/evaluate';
import { describe, expect, it } from 'vitest';

import { mirrorFromSession } from '../../../shared/project-store';
import { reframeSession } from '../../../shared/project-store/testing/reframe-fixture';
import { insideFrame, layerBounds } from './frame-bounds';
import { itemsLeavingFrame, positionalOrigins, sameOrigins } from './out-of-frame';
import {
  adaptationOf,
  defaultClipPolicy,
  evaluableOf,
  formatChanged,
  holdsItems,
  initialFormatValues,
  versionDraft,
  withFormat,
} from './sequence-format';
import { estimateTextLayer, measureTextLayer } from './text-extent';

const hd: Format = {
  width: 1920,
  height: 1080,
  frameRate: { numerator: 25, denominator: 1 },
  sampleRate: 48_000,
};
const vertical: Format = { ...hd, width: 1080, height: 1920 };

const centred = {
  position: { x: 0.5, y: 0.5 },
  scale: { x: 1, y: 1 },
  rotation: 0,
  anchor: { x: 0.5, y: 0.5 },
  opacity: 1,
  blur: 0,
};

describe('frame bounds', () => {
  it('places, scales and rotates boxes around their anchor', () => {
    const bounds = layerBounds(centred, { width: 200, height: 100 }, hd);
    expect(bounds).toEqual({ left: 860, top: 490, right: 1060, bottom: 590 });
    const turned = layerBounds({ ...centred, rotation: 90 }, { width: 200, height: 100 }, hd);
    expect(turned.left).toBeCloseTo(910);
    expect(turned.bottom).toBeCloseTo(640);
    expect(insideFrame(bounds, hd)).toBe(true);
    expect(insideFrame(layerBounds(centred, { width: 2000, height: 10 }, hd), hd)).toBe(false);
  });
});

describe('text extent', () => {
  const layer = (overrides: Partial<TextLayer> = {}): TextLayer =>
    ({
      kind: 'text',
      itemId: 'title',
      transform: centred,
      content: 'aa bb cc',
      font: { family: 'Inter', weight: 600, italic: false },
      fontSize: 100,
      color: { r: 1, g: 1, b: 1, a: 1 },
      letterSpacing: 0,
      lineHeight: 1.2,
      align: 'center',
      maxWidth: null,
      stroke: null,
      shadow: null,
      background: null,
      reveal: 1,
      ...overrides,
    }) satisfies TextLayer;
  const tenPerCharacter = (text: string) => text.length * 10;

  it('wraps at word boundaries within the wrapping width', () => {
    expect(measureTextLayer(layer(), tenPerCharacter)).toEqual({ width: 80, height: 120 });
    expect(measureTextLayer(layer({ maxWidth: 55 }), tenPerCharacter)).toEqual({
      width: 50,
      height: 240,
    });
  });

  it('adds letter spacing, the outline and the plate', () => {
    const size = measureTextLayer(
      layer({
        letterSpacing: 2,
        stroke: { color: { r: 0, g: 0, b: 0, a: 1 }, width: 3 },
        background: {
          color: { r: 0, g: 0, b: 0, a: 1 },
          padding: { x: 10, y: 5 },
          cornerRadius: 0,
        },
      }),
      tenPerCharacter,
    );
    expect(size).toEqual({ width: 80 + 14 + 6 + 20, height: 120 + 6 + 10 });
    expect(estimateTextLayer(layer()).width).toBeCloseTo(8 * 55);
  });
});

describe('sequence format values', () => {
  const mirror = mirrorFromSession(reframeSession());
  const sequence = Object.values(mirror.sequences)[0];
  if (sequence === undefined) throw new Error('fixture has a sequence');

  it('proposes 9:16 for landscape sequences and fills a frame of another aspect', () => {
    expect(versionDraft(hd)).toMatchObject({ width: 1080, height: 1920 });
    expect(versionDraft(vertical)).toMatchObject({ width: 1920, height: 1080 });
    expect(defaultClipPolicy(hd, versionDraft(hd))).toBe('fill');
    expect(defaultClipPolicy(hd, { ...versionDraft(hd), width: 3840, height: 2160 })).toBe('keep');
  });

  it('names versions after their ratio until the name is edited', () => {
    const values = initialFormatValues(sequence, 'version');
    expect(values.name).toBe('Film 9:16');
    const square = withFormat(
      values,
      { ...values, ratio: { width: 1, height: 1 }, width: 1080, height: 1080 },
      'Film',
    );
    expect(square.name).toBe('Film 1:1');
    const edited = { ...square, name: 'Short', nameEdited: true };
    expect(withFormat(edited, values, 'Film').name).toBe('Short');
    expect(adaptationOf(values, hd)).toEqual({ clipReframe: 'fill', fitTextWidths: true });
    expect(adaptationOf({ ...values, clipPolicy: 'fit' }, hd).clipReframe).toBe('fit');
  });

  it('knows when a sequence holds items and when the format changed', () => {
    expect(holdsItems({ sequence, tracks: mirror.tracks })).toBe(true);
    const current = initialFormatValues(sequence, 'format');
    expect(formatChanged(current, sequence.format)).toBe(false);
    expect(formatChanged({ ...current, width: 1080, height: 1920 }, sequence.format)).toBe(true);
  });
});

describe('items leaving the frame', () => {
  const mirror = mirrorFromSession(reframeSession());
  const sequence = Object.values(mirror.sequences)[0];
  if (sequence === undefined) throw new Error('fixture has a sequence');
  const before = evaluableOf({ sequence, tracks: mirror.tracks, items: mirror.items });
  const measureText = estimateTextLayer;

  it('reports what a narrower frame pushes out, not clips that fill or fit it', () => {
    const after = { ...before, format: vertical };
    const leaving = itemsLeavingFrame({
      before,
      after,
      assets: mirror.assets,
      origins: sameOrigins(after),
      measureText,
    });
    expect(leaving).toEqual(['title']);
  });

  it('ignores items already outside before the change', () => {
    const offscreen = withTitle(mirror.items, { x: -0.5, y: 0.5 });
    const shifted = evaluableOf({ sequence, tracks: mirror.tracks, items: offscreen });
    const leaving = itemsLeavingFrame({
      before: shifted,
      after: { ...shifted, format: vertical },
      assets: mirror.assets,
      origins: sameOrigins(shifted),
      measureText,
    });
    expect(leaving).toEqual([]);
  });

  it('pairs a copy with its source by position', () => {
    const origins = positionalOrigins(before, before);
    expect([...origins.entries()]).toEqual([
      ['wide', 'wide'],
      ['filled', 'filled'],
      ['title', 'title'],
    ]);
  });
});

/** The fixture's items with the title moved to `position`. */
function withTitle(
  items: Readonly<Record<string, ItemEntity>>,
  position: { x: number; y: number },
): Record<string, ItemEntity> {
  const title = items['title'];
  if (title?.kind.kind !== 'text') throw new Error('fixture has a title');
  const moved: ItemEntity = {
    ...title,
    kind: {
      ...title.kind,
      transform: { ...title.kind.transform, position: { kind: 'static', value: position } },
    },
  };
  return { ...items, title: moved };
}
