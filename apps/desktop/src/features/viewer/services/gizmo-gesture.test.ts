import type { Animatable, Flicks, ItemEntity } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { gestureStep, stepAnimations, type GizmoGesture } from './gizmo-gesture';
import { documentColor } from '../../../shared/project-store/testing/graphics-fixture';

const frame = { width: 1920, height: 1080 };
const box = { width: 400, height: 200 };
const placement = {
  position: { x: 0.5, y: 0.5 },
  scale: { x: 1, y: 1 },
  rotation: 0,
  anchor: { x: 0.5, y: 0.5 },
};
const S = 705_600_000;
const still = <T>(value: T): Animatable<T> => ({ kind: 'static', value });

function shape(id: string, position: Animatable<{ x: number; y: number }>): ItemEntity {
  return {
    id,
    trackId: 'v1',
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: S as Flicks, duration: (4 * S) as Flicks },
    linkGroup: null,
    kind: {
      kind: 'shape',
      geometry: { kind: 'rectangle', width: 400, height: 200, cornerRadius: 0 },
      fill: still(documentColor('ffffff')),
      stroke: { enabled: false, color: still(documentColor('000000')), width: still(4) },
      coverFrame: false,
      transform: {
        position,
        scale: still({ x: 1, y: 1 }),
        rotation: still(0),
        anchor: still({ x: 0.5, y: 0.5 }),
        opacity: still(1),
        blur: still(0),
      },
    },
  };
}

describe('gizmo gestures', () => {
  const move: GizmoGesture = {
    action: { kind: 'move' },
    from: { x: 960, y: 540 },
    layers: [
      { itemId: 'a', transform: placement, box },
      { itemId: 'b', transform: { ...placement, position: { x: 0.25, y: 0.25 } }, box },
    ],
  };

  it('moves every dragged item, snapping the lead to the frame', () => {
    const step = gestureStep(move, frame, { x: 966, y: 640 }, { shift: false, snapThreshold: 8 });
    expect(step.placements.get('a')?.position.x).toBeCloseTo(0.5);
    expect(step.placements.get('b')?.position.x).toBeCloseTo(0.25);
    expect(step.placements.get('b')?.position.y).toBeCloseTo(0.25 + 100 / 1080);
    expect(step.guides.x).toEqual([960]);
    expect(step.properties).toEqual(['position']);
  });

  it('writes static values and keyframes at the playhead', () => {
    const keyframed = shape('b', {
      kind: 'keyframed',
      keyframes: [{ time: 0 as Flicks, value: { x: 0.25, y: 0.25 }, easing: { kind: 'linear' } }],
    });
    const step = gestureStep(move, frame, { x: 1152, y: 540 }, { shift: false, snapThreshold: 0 });
    const changes = stepAnimations(
      step,
      [shape('a', still(placement.position)), keyframed],
      (3 * S) as Flicks,
      1000,
    );
    expect(changes).toHaveLength(2);
    expect(changes[0]?.animation).toEqual({
      property: 'position',
      value: still({ x: 0.6, y: 0.5 }),
    });
    const second = changes[1]?.animation.value;
    expect(
      second?.kind === 'keyframed' && second.keyframes.map((keyframe) => keyframe.time),
    ).toEqual([0, 2 * S]);
  });

  it('re-anchors by changing anchor and position together', () => {
    const anchor: GizmoGesture = {
      action: { kind: 'anchor' },
      from: { x: 960, y: 540 },
      layers: move.layers,
    };
    const step = gestureStep(anchor, frame, { x: 760, y: 440 }, { shift: false, snapThreshold: 0 });
    expect(step.properties).toEqual(['anchor', 'position']);
    expect(step.placements.get('a')?.anchor.x).toBeCloseTo(0);
    expect(step.placements.has('b')).toBe(false);
  });
});
