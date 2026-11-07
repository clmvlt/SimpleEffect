import type {
  AnimatableProperty,
  Flicks,
  ItemAnimation,
  ItemEntity,
  ItemId,
  Vec2,
} from '@se/protocol';

import { animationOf, propertyOf, withValueAt } from '../../../shared/animation';
import {
  moved,
  reanchored,
  rotated,
  scaled,
  snapToFrame,
  type FrameSize,
  type PlacedLayer,
  type Placement,
  type ScaleHandle,
  type SnapGuides,
} from './gizmo-geometry';

/** What a gizmo drag does. */
export type GizmoAction =
  | { readonly kind: 'move' }
  | { readonly kind: 'scale'; readonly handle: ScaleHandle }
  | { readonly kind: 'rotate' }
  | { readonly kind: 'anchor' };

/** A drag in progress: the action, where it started and the layers as they were then. */
export interface GizmoGesture {
  readonly action: GizmoAction;
  /** Pointer at the start, in frame pixels. */
  readonly from: Vec2;
  /** The dragged layers at the start; the first one leads (snapping, scale handles). */
  readonly layers: readonly PlacedLayer[];
}

/** Modifier keys and snapping of a gesture step. */
export interface GestureOptions {
  /** Shift: proportional scale, rotation by 15°. */
  readonly shift: boolean;
  /** Snap distance in frame pixels (0 disables snapping). */
  readonly snapThreshold: number;
}

/** The placements a gesture step gives each dragged item, the properties it changes and guides. */
export interface GestureStep {
  readonly placements: ReadonlyMap<ItemId, Placement>;
  readonly properties: readonly AnimatableProperty[];
  readonly guides: SnapGuides;
}

const NO_GUIDES: SnapGuides = { x: [], y: [] };

/** The placements of `gesture` with the pointer at `to` (frame pixels). */
export function gestureStep(
  gesture: GizmoGesture,
  frame: FrameSize,
  to: Vec2,
  options: GestureOptions,
): GestureStep {
  const [lead] = gesture.layers;
  const placements = new Map<ItemId, Placement>();
  if (lead === undefined) return { placements, properties: [], guides: NO_GUIDES };
  const { action } = gesture;
  if (action.kind === 'move') {
    const delta = { x: to.x - gesture.from.x, y: to.y - gesture.from.y };
    const leadMoved = moved(lead.transform, frame, delta);
    const snap =
      options.snapThreshold > 0
        ? snapToFrame(leadMoved, lead.box, frame, options.snapThreshold)
        : { placement: leadMoved, guides: NO_GUIDES };
    const shift = {
      x: (snap.placement.position.x - leadMoved.position.x) * frame.width,
      y: (snap.placement.position.y - leadMoved.position.y) * frame.height,
    };
    for (const layer of gesture.layers) {
      placements.set(
        layer.itemId,
        moved(layer.transform, frame, { x: delta.x + shift.x, y: delta.y + shift.y }),
      );
    }
    return { placements, properties: ['position'], guides: snap.guides };
  }
  if (action.kind === 'scale') {
    placements.set(
      lead.itemId,
      scaled(lead.transform, lead.box, frame, action.handle, to, options.shift),
    );
    return { placements, properties: ['scale'], guides: NO_GUIDES };
  }
  if (action.kind === 'rotate') {
    placements.set(lead.itemId, rotated(lead.transform, frame, gesture.from, to, options.shift));
    return { placements, properties: ['rotation'], guides: NO_GUIDES };
  }
  placements.set(lead.itemId, reanchored(lead.transform, lead.box, frame, to));
  return { placements, properties: ['anchor', 'position'], guides: NO_GUIDES };
}

/**
 * The property values a step sets on the document's items at the playhead: static properties take
 * the value, keyframed ones get a keyframe there (`withValueAt`).
 */
export function stepAnimations(
  step: GestureStep,
  items: readonly ItemEntity[],
  playhead: Flicks,
  tolerance: number,
): ItemAnimation[] {
  const changes: ItemAnimation[] = [];
  for (const item of items) {
    const placement = step.placements.get(item.id);
    if (placement === undefined) continue;
    const local = Math.min(
      item.timelineRange.duration,
      Math.max(0, playhead - item.timelineRange.start),
    ) as Flicks;
    for (const property of step.properties) {
      if (property === 'position' || property === 'scale' || property === 'anchor') {
        const current = propertyOf(item.kind, property);
        if (current === null) continue;
        const value = withValueAt(current, local, placement[property], tolerance);
        changes.push({ itemId: item.id, animation: animationOf(property, value) });
      } else if (property === 'rotation') {
        const current = propertyOf(item.kind, 'rotation');
        if (current === null) continue;
        const value = withValueAt(current, local, placement.rotation, tolerance);
        changes.push({ itemId: item.id, animation: animationOf('rotation', value) });
      }
    }
  }
  return changes;
}
