import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { allSame, keepAlpha, keyframeTolerance, localTime, opaquePart } from './inspector-values';
import { documentColor } from '../../../shared/project-store/testing/graphics-fixture';

const S = 705_600_000;

describe('inspector values', () => {
  it('clamps the playhead into the item', () => {
    const item = { timelineRange: { start: S as Flicks, duration: (2 * S) as Flicks } };
    expect(localTime(item, (S / 2) as Flicks)).toBe(0);
    expect(localTime(item, (2 * S) as Flicks)).toBe(S);
    expect(localTime(item, (9 * S) as Flicks)).toBe(2 * S);
  });

  it('stands on a keyframe within half a frame', () => {
    expect(keyframeTolerance({ numerator: 25, denominator: 1 })).toBe(S / 50);
  });

  it('edits opaque colours and keeps the alpha', () => {
    expect(opaquePart(documentColor('000000B3'))).toBe(documentColor('000000'));
    expect(keepAlpha(documentColor('ff0000'), documentColor('000000b3'))).toBe(
      documentColor('ff0000b3'),
    );
    expect(keepAlpha(documentColor('ff0000'), documentColor('00ff00'))).toBe(
      documentColor('ff0000'),
    );
    expect(allSame([1, 1, 1], (a, b) => a === b)).toBe(true);
    expect(allSame([1, 2], (a, b) => a === b)).toBe(false);
  });
});
