import { BlurFilter, type Container } from 'pixi.js';

import type { Rgba } from '../../animation/color';
import type { Layer, Size } from '../../evaluate/frame-graph';
import { layerPlacement } from '../layout';

/**
 * The scene node of one layer, reused across renders while the same item keeps the same kind:
 * a placement container holding the layer's drawing.
 */
export interface LayerNode {
  readonly kind: Layer['kind'];
  readonly root: Container;
  /** Frees the node and the GPU resources it owns (never the pictures: the pool owns them). */
  destroy(): void;
}

/** Pixi colour number and alpha of a colour. */
export function pixiColor(color: Rgba): { readonly color: number; readonly alpha: number } {
  const byte = (value: number) => Math.round(Math.min(1, Math.max(0, value)) * 255);
  return { color: (byte(color.r) << 16) | (byte(color.g) << 8) | byte(color.b), alpha: color.a };
}

/** Places `root` (holding a drawing of `box` size) with the layer's transform. */
export function place(root: Container, layer: Layer, box: Size, frame: Size): void {
  const placement = layerPlacement(layer.transform, box, frame);
  root.position.set(placement.x, placement.y);
  root.pivot.set(placement.pivotX, placement.pivotY);
  root.scale.set(placement.scaleX, placement.scaleY);
  root.rotation = placement.rotation;
  root.alpha = placement.alpha;
}

/** Below this radius (output pixels) a blur is invisible: the filter is left off. */
const MIN_BLUR = 0.05;

/**
 * The blur of one layer: a Pixi blur filter on its root while its radius is visible, removed
 * otherwise (filters render the layer offscreen: never keep one for nothing).
 */
export class LayerBlur {
  private filter: BlurFilter | null = null;
  /** Whether the filter is on the root (Pixi leaves `filters` unset until assigned). */
  private attached = false;

  /** Blurs `root` by `radius` output pixels (0 removes the filter). */
  apply(root: Container, radius: number): void {
    if (radius < MIN_BLUR) {
      if (this.attached) root.filters = [];
      this.attached = false;
      return;
    }
    if (this.filter === null) this.filter = new BlurFilter({ strength: radius, quality: 4 });
    else this.filter.strength = radius;
    if (!this.attached) root.filters = [this.filter];
    this.attached = true;
  }

  destroy(): void {
    this.filter?.destroy();
    this.filter = null;
  }
}
