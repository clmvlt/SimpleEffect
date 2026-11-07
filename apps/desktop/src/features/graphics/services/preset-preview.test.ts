import type { AnimationPresetSample, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { previewStyle } from './preset-preview';

const SECOND = 705_600_000;
const linear = { kind: 'linear' } as const;
const slide: AnimationPresetSample = {
  phase: 'in',
  animations: [
    {
      property: 'position',
      value: {
        kind: 'keyframed',
        keyframes: [
          { time: 0 as Flicks, value: { x: 0.4, y: 0.5 }, easing: linear },
          { time: SECOND as Flicks, value: { x: 0.5, y: 0.5 }, easing: linear },
        ],
      },
    },
    {
      property: 'opacity',
      value: {
        kind: 'keyframed',
        keyframes: [
          { time: 0 as Flicks, value: 0, easing: linear },
          { time: SECOND as Flicks, value: 1, easing: linear },
        ],
      },
    },
  ],
};

describe('preset previews', () => {
  it('map the sample onto the swatch', () => {
    const start = previewStyle(slide, 0 as Flicks, { width: 100, height: 60 });
    expect(start.transform).toBe('translate(-10px, 0px) rotate(0deg) scale(1, 1)');
    expect(start.opacity).toBe(0);
    const middle = previewStyle(slide, (SECOND / 2) as Flicks, { width: 100, height: 60 });
    expect(middle.transform).toBe('translate(-5px, 0px) rotate(0deg) scale(1, 1)');
    expect(middle.opacity).toBe(0.5);
    expect(middle.clipPath).toBe('inset(0 0% 0 0)');
  });

  it('types text out and blurs in CSS pixels', () => {
    const typed: AnimationPresetSample = {
      phase: 'in',
      animations: [
        {
          property: 'reveal',
          value: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: 0, easing: linear },
              { time: SECOND as Flicks, value: 1, easing: linear },
            ],
          },
        },
        { property: 'blur', value: { kind: 'static', value: 40 } },
      ],
    };
    const style = previewStyle(typed, (SECOND / 4) as Flicks, { width: 100, height: 60 });
    expect(style.clipPath).toBe('inset(0 75% 0 0)');
    expect(style.filter).toBe('blur(2px)');
  });
});
