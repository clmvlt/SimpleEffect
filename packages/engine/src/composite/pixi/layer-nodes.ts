import { Container, Graphics, ImageSource, Sprite, Text, Texture } from 'pixi.js';

import type { Rgba } from '../../animation/color';
import type { Layer, ShapeLayer, Size, TextLayer } from '../../evaluate/frame-graph';
import type { Picture } from '../compositor';
import { layerPlacement, pictureFit } from '../layout';

/** Fonts tried after the layer's family: the bundled UI font, then the platform's sans-serif. */
const FALLBACK_FONTS = ['Inter Variable', 'sans-serif'];

/** Pixel size Pixi reads from a picture (`displayWidth` for frames, `width` for bitmaps). */
function resourceSize(resource: VideoFrame | ImageBitmap): readonly [number, number] {
  return 'displayWidth' in resource
    ? [resource.displayWidth, resource.displayHeight]
    : [resource.width, resource.height];
}

/** Pixi colour number and alpha of a colour. */
function pixiColor(color: Rgba): { readonly color: number; readonly alpha: number } {
  const byte = (value: number) => Math.round(Math.min(1, Math.max(0, value)) * 255);
  return { color: (byte(color.r) << 16) | (byte(color.g) << 8) | byte(color.b), alpha: color.a };
}

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

function place(root: Container, layer: Layer, box: Size, frame: Size): void {
  const placement = layerPlacement(layer.transform, box, frame);
  root.position.set(placement.x, placement.y);
  root.pivot.set(placement.pivotX, placement.pivotY);
  root.scale.set(placement.scaleX, placement.scaleY);
  root.rotation = placement.rotation;
  root.alpha = placement.alpha;
}

/** A video frame or an image stretched over the layer's box. */
export class PictureNode implements LayerNode {
  readonly kind;
  readonly root = new Container();
  private readonly sprite = new Sprite();
  private source: ImageSource | null = null;

  constructor(kind: 'video' | 'image') {
    this.kind = kind;
    this.sprite.anchor.set(0.5);
    this.root.addChild(this.sprite);
  }

  /** Shows `picture` with the layer's placement; hides the sprite when there is none. */
  update(layer: Layer & { readonly box: Size }, picture: Picture | undefined, frame: Size): void {
    place(this.root, layer, layer.box, frame);
    this.sprite.visible = picture !== undefined;
    if (picture === undefined) return;
    this.upload(picture.source);
    const fit = pictureFit(layer.box, picture.rotation, picture.mirrored);
    const texture = this.sprite.texture;
    this.sprite.position.set(fit.x, fit.y);
    this.sprite.rotation = fit.rotation;
    this.sprite.scale.set(
      (fit.width / texture.width) * (fit.mirrored ? -1 : 1),
      fit.height / texture.height,
    );
  }

  /**
   * Points the texture at new pixels (uploaded during the render that follows at once). Pixels of
   * the same size reuse the GPU texture; another size (the proxy toggled, a new clip on the item)
   * gets a new texture: resizing a texture in place left its frame at the old size.
   */
  private upload(resource: VideoFrame | ImageBitmap): void {
    if (this.source?.resource === resource) return;
    const [width, height] = resourceSize(resource);
    if (this.source?.pixelWidth === width && this.source.pixelHeight === height) {
      this.source.resource = resource;
      this.source.update();
      return;
    }
    this.releaseTexture();
    this.source = new ImageSource({ resource, autoGarbageCollect: false });
    this.sprite.texture = new Texture({ source: this.source });
  }

  private releaseTexture(): void {
    if (this.source === null) return;
    const texture = this.sprite.texture;
    this.sprite.texture = Texture.EMPTY;
    texture.destroy(true);
    this.source = null;
  }

  destroy(): void {
    this.releaseTexture();
    this.root.destroy({ children: true });
  }
}

/** A filled rectangle (rounded) or ellipse. */
export class ShapeNode implements LayerNode {
  readonly kind = 'shape';
  readonly root = new Container();
  private readonly graphics = new Graphics();

  constructor() {
    this.root.addChild(this.graphics);
  }

  update(layer: ShapeLayer, frame: Size): void {
    place(this.root, layer, layer.box, frame);
    const { width, height } = layer.box;
    this.graphics.clear();
    if (layer.geometry === 'ellipse') {
      this.graphics.ellipse(width / 2, height / 2, width / 2, height / 2);
    } else if (layer.cornerRadius > 0) {
      this.graphics.roundRect(0, 0, width, height, layer.cornerRadius);
    } else {
      this.graphics.rect(0, 0, width, height);
    }
    this.graphics.fill(pixiColor(layer.fill));
  }

  destroy(): void {
    this.root.destroy({ children: true });
  }
}

/**
 * Basic text: font family, size and colour, anchored in its measured box.
 * TODO(prompt-15): full text styles (weight, alignment, stroke, shadow…) and animations.
 */
export class TextNode implements LayerNode {
  readonly kind = 'text';
  readonly root = new Container();
  private readonly text = new Text({ text: '' });

  constructor() {
    this.root.addChild(this.text);
  }

  /** `resolution` rasterises glyphs at the output scale so they stay sharp when zoomed. */
  update(layer: TextLayer, frame: Size, resolution: number): void {
    const { color, alpha } = pixiColor(layer.color);
    this.text.text = layer.content;
    this.text.style.fontFamily = [layer.fontFamily, ...FALLBACK_FONTS];
    this.text.style.fontSize = layer.fontSize;
    this.text.style.fill = color;
    this.text.alpha = alpha;
    this.text.resolution = resolution;
    place(this.root, layer, { width: this.text.width, height: this.text.height }, frame);
  }

  destroy(): void {
    this.root.destroy({ children: true });
  }
}
