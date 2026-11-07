import type { Animatable, Flicks, ItemEntity } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  animatedProperties,
  deletedKeyframeAnimations,
  easedKeyframeAnimation,
  keyframeLaneHeight,
  movedKeyframeAnimations,
} from './keyframe-lane';
import { documentColor } from '../../../shared/project-store/testing/graphics-fixture';

const S = 705_600_000;
const still = <T>(value: T): Animatable<T> => ({ kind: 'static', value });
const linear = { kind: 'linear' } as const;
const opacity: Animatable<number> = {
  kind: 'keyframed',
  keyframes: [
    { time: 0 as Flicks, value: 0, easing: linear },
    { time: S as Flicks, value: 1, easing: linear },
  ],
};

const title: ItemEntity = {
  id: 'title',
  trackId: 'v1',
  revision: 1,
  origin: { kind: 'user' },
  timelineRange: { start: 0 as Flicks, duration: (3 * S) as Flicks },
  linkGroup: null,
  kind: {
    kind: 'shape',
    geometry: { kind: 'ellipse', width: 10, height: 10 },
    fill: still(documentColor('ffffff')),
    stroke: { enabled: false, color: still(documentColor('000000')), width: still(4) },
    coverFrame: false,
    transform: {
      position: still({ x: 0.5, y: 0.5 }),
      scale: still({ x: 1, y: 1 }),
      rotation: { kind: 'keyframed', keyframes: [{ time: 0 as Flicks, value: 0, easing: linear }] },
      anchor: still({ x: 0.5, y: 0.5 }),
      opacity,
      blur: still(0),
    },
  },
};

describe('keyframe lanes', () => {
  it('have one row per animated property', () => {
    expect(animatedProperties(title.kind)).toEqual(['rotation', 'opacity']);
    expect(keyframeLaneHeight(title.kind)).toBe(32);
  });

  it('move selected keyframes in one change per property, refusing collisions', () => {
    const keys = [{ property: 'opacity' as const, time: S as Flicks }];
    const moved = movedKeyframeAnimations(title, keys, S / 2);
    const value = moved?.[0]?.animation.value;
    expect(value?.kind === 'keyframed' && value.keyframes.map((keyframe) => keyframe.time)).toEqual(
      [0, 1.5 * S],
    );
    expect(movedKeyframeAnimations(title, keys, -S)).toBeNull();
  });

  it('delete keyframes and change easings', () => {
    const keys = [{ property: 'opacity' as const, time: 0 as Flicks }];
    expect(deletedKeyframeAnimations(title, keys)[0]?.animation.value).toEqual({
      kind: 'keyframed',
      keyframes: [{ time: S, value: 1, easing: linear }],
    });
    const eased = easedKeyframeAnimation(
      title,
      keys[0] ?? { property: 'opacity', time: 0 as Flicks },
      {
        kind: 'hold',
      },
    );
    const value = eased?.animation.value;
    expect(value?.kind === 'keyframed' && value.keyframes[0]?.easing).toEqual({ kind: 'hold' });
  });
});
