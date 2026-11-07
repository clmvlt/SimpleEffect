import type { Animatable, Color, Easing, Flicks, Vec2 } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import vectorsText from '../../../../docs/test-vectors/interpolation.json?raw';
import { parseColor, rgbaToOklab, formatColor, mixColors } from './color';
import { cubicBezierProgress, type CubicBezierCurve } from './cubic-bezier';
import { easedProgress } from './easing';
import { interpolateColor, interpolateNumber, interpolateVec2, valueAt } from './interpolate';

interface KeyframeCase {
  readonly valueKind: 'number' | 'vec2' | 'color';
  readonly property: Animatable<unknown>;
  readonly samples: readonly { readonly time: Flicks; readonly value: unknown }[];
}

interface InterpolationVectors {
  readonly tolerance: number;
  readonly cubicBezier: readonly {
    readonly curve: CubicBezierCurve;
    readonly samples: readonly { readonly x: number; readonly y: number }[];
  }[];
  readonly easings: readonly {
    readonly easing: Easing;
    readonly samples: readonly { readonly progress: number; readonly eased: number }[];
  }[];
  readonly oklab: readonly {
    readonly color: Color;
    readonly l: number;
    readonly a: number;
    readonly b: number;
  }[];
  readonly colorMixes: readonly {
    readonly from: Color;
    readonly to: Color;
    readonly progress: number;
    readonly expected: Color;
  }[];
  readonly keyframes: readonly KeyframeCase[];
}

const vectors = JSON.parse(vectorsText) as InterpolationVectors;
const digits = -Math.log10(vectors.tolerance);

function evaluateCase(testCase: KeyframeCase, time: Flicks): unknown {
  switch (testCase.valueKind) {
    case 'number':
      return valueAt(testCase.property as Animatable<number>, time, interpolateNumber, Number.NaN);
    case 'vec2':
      return valueAt(testCase.property as Animatable<Vec2>, time, interpolateVec2, {
        x: Number.NaN,
        y: Number.NaN,
      });
    case 'color':
      return valueAt(testCase.property as Animatable<Color>, time, interpolateColor, '');
  }
}

describe('shared interpolation vectors (docs/test-vectors/interpolation.json)', () => {
  it.each(vectors.cubicBezier)('solves cubic-bezier %j', ({ curve, samples }) => {
    for (const sample of samples) {
      expect(cubicBezierProgress(curve, sample.x)).toBeCloseTo(sample.y, digits);
    }
  });

  it.each(vectors.easings)('eases with %j', ({ easing, samples }) => {
    for (const sample of samples) {
      expect(easedProgress(easing, sample.progress)).toBeCloseTo(sample.eased, digits);
    }
  });

  it.each(vectors.oklab)('converts $color to OKLab', ({ color, l, a, b }) => {
    const parsed = parseColor(color);
    expect(parsed).not.toBeNull();
    if (parsed === null) return;
    const oklab = rgbaToOklab(parsed);
    expect(oklab.l).toBeCloseTo(l, digits);
    expect(oklab.a).toBeCloseTo(a, digits);
    expect(oklab.b).toBeCloseTo(b, digits);
  });

  it.each(vectors.colorMixes)('mixes $from → $to at $progress', (mix) => {
    const from = parseColor(mix.from);
    const to = parseColor(mix.to);
    if (from === null || to === null) throw new Error('malformed vector colour');
    expect(formatColor(mixColors(from, to, mix.progress))).toBe(mix.expected);
  });

  it.each(vectors.keyframes)('evaluates a $valueKind property', (testCase) => {
    for (const sample of testCase.samples) {
      const actual = evaluateCase(testCase, sample.time);
      if (typeof sample.value === 'number') {
        expect(actual).toBeCloseTo(sample.value, digits);
      } else if (typeof sample.value === 'string') {
        expect(actual).toBe(sample.value);
      } else {
        const expected = sample.value as Vec2;
        const vector = actual as Vec2;
        expect(vector.x).toBeCloseTo(expected.x, digits);
        expect(vector.y).toBeCloseTo(expected.y, digits);
      }
    }
  });
});
