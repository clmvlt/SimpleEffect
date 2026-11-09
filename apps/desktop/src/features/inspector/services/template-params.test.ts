import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { SECOND, documentColor } from '../../../shared/project-store/testing/graphics-fixture';
import { badgeTemplate } from '../../../shared/project-store/testing/template-fixture';
import {
  colorAt,
  numberAt,
  paramKeyframeState,
  paramValue,
  toggledParamKeyframe,
  withNumberAt,
} from './template-params';

const [label, size, tint] = badgeTemplate().params;

describe('template parameter values', () => {
  it('uses the instance’s value of the right kind, else the default', () => {
    if (label === undefined || size === undefined) throw new Error('fixture');
    expect(paramValue(label, { label: { kind: 'text', value: 'PROMO' } })).toEqual({
      kind: 'text',
      value: 'PROMO',
    });
    expect(paramValue(label, { label: { kind: 'boolean', value: true } })).toEqual({
      kind: 'text',
      value: 'NEW',
    });
    expect(numberAt(paramValue(size, {}), 0 as Flicks)).toBe(60);
    if (tint !== undefined) {
      expect(colorAt(paramValue(tint, {}), 0 as Flicks)).toBe(documentColor('ff3366'));
    }
  });

  it('keyframes numbers at the playhead and interpolates between keyframes', () => {
    if (size === undefined) throw new Error('fixture');
    const keyed = toggledParamKeyframe(paramValue(size, {}), 0 as Flicks, 1);
    expect(paramKeyframeState(keyed, 0 as Flicks, 1)).toBe('onKeyframe');
    const two = withNumberAt(keyed, (2 * SECOND) as Flicks, 160, 1);
    expect(numberAt(two, SECOND)).toBe(110);
    expect(paramKeyframeState(two, SECOND, 1)).toBe('betweenKeyframes');
    const untouched = withNumberAt(paramValue(size, {}), SECOND, 90, 1);
    expect(untouched).toEqual({ kind: 'number', value: { kind: 'static', value: 90 } });
  });
});
