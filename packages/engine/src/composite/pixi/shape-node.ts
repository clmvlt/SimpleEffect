import { Container, Graphics } from 'pixi.js';

import type { ShapeLayer, Size } from '../../evaluate/frame-graph';
import { LayerBlur, pixiColor, place, type LayerNode } from './layer-node';

/** Pixi stroke alignment drawing entirely outside the path. */
const OUTSIDE = 0;

/** A filled rectangle (rounded), ellipse or line capsule, with an optional outline outside. */
export class ShapeNode implements LayerNode {
  readonly kind = 'shape';
  readonly root = new Container();
  private readonly graphics = new Graphics();
  private readonly blur = new LayerBlur();
  /** What the graphics currently draw: redrawn only when it changes. */
  private drawn = '';

  constructor() {
    this.root.addChild(this.graphics);
  }

  /** Draws `layer`; `scale` is output pixels per sequence pixel. Returns the layer's box. */
  update(layer: ShapeLayer, frame: Size, scale: number): Size {
    place(this.root, layer, layer.box, frame);
    this.blur.apply(this.root, layer.transform.blur * scale);
    const key = JSON.stringify([
      layer.geometry,
      layer.box,
      layer.cornerRadius,
      layer.fill,
      layer.stroke,
    ]);
    if (key !== this.drawn) {
      this.drawn = key;
      this.draw(layer);
    }
    return layer.box;
  }

  private draw(layer: ShapeLayer): void {
    const { width, height } = layer.box;
    const graphics = this.graphics;
    graphics.clear();
    if (layer.geometry === 'ellipse') {
      graphics.ellipse(width / 2, height / 2, width / 2, height / 2);
    } else if (layer.cornerRadius > 0) {
      graphics.roundRect(0, 0, width, height, Math.min(layer.cornerRadius, width / 2, height / 2));
    } else {
      graphics.rect(0, 0, width, height);
    }
    graphics.fill(pixiColor(layer.fill));
    if (layer.stroke !== null) {
      graphics.stroke({
        ...pixiColor(layer.stroke.color),
        width: layer.stroke.width,
        alignment: OUTSIDE,
      });
    }
  }

  destroy(): void {
    this.blur.destroy();
    this.root.destroy({ children: true });
  }
}
