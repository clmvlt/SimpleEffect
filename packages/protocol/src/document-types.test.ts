import { describe, expectTypeOf, it } from 'vitest';

import type {
  Animatable,
  Flicks,
  ItemKind,
  MediaClip,
  Project,
  Sequence,
  TimeRange,
  ViolationCode,
} from './generated';

// Compile-time checks of the document types generated from se-domain (`tsc -b` in `pnpm check`).
describe('generated document types', () => {
  it('brands flicks so plain numbers need an explicit conversion', () => {
    expectTypeOf<Flicks>().toExtend<number>();
    expectTypeOf<number>().not.toExtend<Flicks>();
    expectTypeOf<TimeRange>().toEqualTypeOf<{ start: Flicks; duration: Flicks }>();
  });

  it('discriminates item kinds and animatable properties by `kind`', () => {
    expectTypeOf<ItemKind['kind']>().toEqualTypeOf<
      'mediaClip' | 'text' | 'shape' | 'image' | 'motionTemplate' | 'caption'
    >();
    expectTypeOf<Extract<ItemKind, { kind: 'mediaClip' }>>().toExtend<MediaClip>();
    expectTypeOf<
      Extract<Animatable<number>, { kind: 'static' }>['value']
    >().toEqualTypeOf<number>();
  });

  it('exposes the whole document from the project root', () => {
    expectTypeOf<Project['sequences']>().toEqualTypeOf<Sequence[]>();
    expectTypeOf<'OVERLAPPING_ITEMS'>().toExtend<ViolationCode>();
  });
});
