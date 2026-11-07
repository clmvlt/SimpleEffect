import type { Animatable, Flicks, ItemEntity, ItemKind } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  animationOf,
  clearPropertyPreview,
  curveToEasing,
  easingToCurve,
  hasProperty,
  interpolatorOf,
  keyframeState,
  movedKeyframes,
  neighbourKeyframe,
  previewAnimations,
  previewedItem,
  propertyOf,
  propertyPreviewStore,
  toggledKeyframe,
  valueAtTime,
  withAnimation,
  withValueAt,
  withoutKeyframes,
} from '.';
import { documentColor } from '../project-store/testing/graphics-fixture';

const S = 705_600_000;
const at = (seconds: number) => Math.round(seconds * S) as Flicks;
const still = <T>(value: T): Animatable<T> => ({ kind: 'static', value });
const linear = { kind: 'linear' } as const;
const keyed = (points: [number, number][]): Animatable<number> => ({
  kind: 'keyframed',
  keyframes: points.map(([time, value]) => ({ time: at(time), value, easing: linear })),
});

const transform = {
  position: still({ x: 0.5, y: 0.5 }),
  scale: still({ x: 1, y: 1 }),
  rotation: still(0),
  anchor: still({ x: 0.5, y: 0.5 }),
  opacity: keyed([
    [0, 0],
    [1, 1],
  ]),
  blur: still(0),
};

const shape: ItemKind = {
  kind: 'shape',
  geometry: { kind: 'ellipse', width: 100, height: 100 },
  fill: still(documentColor('ffffff')),
  stroke: { enabled: false, color: still(documentColor('000000')), width: still(4) },
  coverFrame: false,
  transform,
};

describe('property access', () => {
  it('reads the properties an item kind has', () => {
    expect(propertyOf(shape, 'fillColor')).toEqual(still(documentColor('ffffff')));
    expect(propertyOf(shape, 'strokeWidth')).toEqual(still(4));
    expect(propertyOf(shape, 'fontSize')).toBeNull();
    expect(hasProperty(shape, 'reveal')).toBe(false);
    expect(hasProperty({ kind: 'caption', text: 'x' }, 'opacity')).toBe(false);
  });

  it('applies an animation to a copy of the item kind', () => {
    const red = withAnimation(shape, animationOf('fillColor', still(documentColor('ff0000'))));
    expect(propertyOf(red, 'fillColor')).toEqual(still(documentColor('ff0000')));
    expect(propertyOf(shape, 'fillColor')).toEqual(still(documentColor('ffffff')));
    const moved = withAnimation(shape, animationOf('position', still({ x: 0.1, y: 0.2 })));
    expect(propertyOf(moved, 'position')).toEqual(still({ x: 0.1, y: 0.2 }));
  });
});

describe('keyframe edits', () => {
  const opacity = keyed([
    [0, 0],
    [1, 1],
  ]);
  const tolerance = at(0.02);

  it('tells where the playhead stands', () => {
    expect(keyframeState(still(1), at(0.5), tolerance)).toBe('none');
    expect(keyframeState(opacity, at(0.5), tolerance)).toBe('betweenKeyframes');
    expect(keyframeState(opacity, at(1.01), tolerance)).toBe('onKeyframe');
  });

  it('auto-keys a keyframed property and replaces a static one', () => {
    expect(withValueAt(still(1), at(0.5), 0.3, tolerance)).toEqual(still(0.3));
    const added = withValueAt(opacity, at(0.5), 0.2, tolerance);
    expect(added.kind === 'keyframed' && added.keyframes.map((k) => k.value)).toEqual([0, 0.2, 1]);
    const changed = withValueAt(opacity, at(1), 0.7, tolerance);
    expect(changed.kind === 'keyframed' && changed.keyframes.map((k) => k.value)).toEqual([0, 0.7]);
  });

  it('toggles keyframes with the interpolated value', () => {
    const number = interpolatorOf('opacity');
    const toggled = toggledKeyframe(opacity, at(0.5), tolerance, number);
    expect(toggled.kind === 'keyframed' && toggled.keyframes[1]?.value).toBeCloseTo(0.5);
    const removed = toggledKeyframe(toggled, at(0.5), tolerance, number);
    expect(removed).toEqual(opacity);
    expect(toggledKeyframe(still(0.4), at(2), tolerance, number)).toEqual({
      kind: 'keyframed',
      keyframes: [{ time: at(2), value: 0.4, easing: linear }],
    });
    const last = keyed([[1, 0.6]]);
    expect(toggledKeyframe(last, at(1), tolerance, number)).toEqual(still(0.6));
  });

  it('navigates, moves and removes keyframes', () => {
    expect(neighbourKeyframe(opacity, at(0.5), 1, tolerance)).toBe(at(1));
    expect(neighbourKeyframe(opacity, at(0), -1, tolerance)).toBeNull();
    const moved = movedKeyframes(opacity, [at(1)], at(5), at(2));
    expect(moved?.kind === 'keyframed' && moved.keyframes[1]?.time).toBe(at(2));
    expect(movedKeyframes(opacity, [at(0)], at(1), at(2))).toBeNull();
    expect(withoutKeyframes(opacity, [at(0), at(1)])).toEqual(still(0));
    expect(valueAtTime(opacity, at(0.25), interpolatorOf('opacity'))).toBeCloseTo(0.25);
  });

  it('maps easings to editor curves and back', () => {
    expect(easingToCurve({ kind: 'preset', name: 'easeOut' })).toEqual({
      kind: 'cubicBezier',
      x1: 0,
      y1: 0,
      x2: 0.58,
      y2: 1,
    });
    expect(curveToEasing(easingToCurve(linear))).toEqual(linear);
    expect(curveToEasing({ kind: 'hold' })).toEqual({ kind: 'hold' });
  });
});

describe('property previews', () => {
  it('draw items with the values being dragged until cleared', () => {
    const item = {
      id: 'shape-1',
      trackId: 'v1',
      revision: 1,
      origin: { kind: 'user' },
      timelineRange: { start: 0 as Flicks, duration: at(2) },
      linkGroup: null,
      kind: shape,
    } satisfies ItemEntity;
    previewAnimations([{ itemId: item.id, animation: animationOf('rotation', still(45)) }]);
    const previewed = previewedItem(item, propertyPreviewStore.getState());
    expect(propertyOf(previewed.kind, 'rotation')).toEqual(still(45));
    clearPropertyPreview();
    expect(previewedItem(item, propertyPreviewStore.getState())).toBe(item);
  });
});
