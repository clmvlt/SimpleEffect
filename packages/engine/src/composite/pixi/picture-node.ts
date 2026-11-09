import { Container, ImageSource, Rectangle, Sprite, Texture } from 'pixi.js';

import type { Crop, Layer, Size } from '../../evaluate/frame-graph';
import type { Picture } from '../compositor';
import { codedCrop, pictureFit } from '../layout';
import { LayerBlur, place, type LayerNode } from './layer-node';

/** Pixel size Pixi reads from a picture (`displayWidth` for frames, `width` for bitmaps). */
function resourceSize(resource: VideoFrame | ImageBitmap): readonly [number, number] {
  return 'displayWidth' in resource
    ? [resource.displayWidth, resource.displayHeight]
    : [resource.width, resource.height];
}

const WHOLE: Crop = { x: 0, y: 0, width: 1, height: 1 };

/**
 * A video frame, an image (the part of it the layer crops) or a motion template's rendered frame,
 * stretched over the layer's box.
 */
export class PictureNode implements LayerNode {
  readonly kind;
  readonly root = new Container();
  private readonly sprite = new Sprite();
  private readonly blur = new LayerBlur();
  private source: ImageSource | null = null;

  constructor(kind: 'video' | 'image' | 'template') {
    this.kind = kind;
    this.sprite.anchor.set(0.5);
    this.root.addChild(this.sprite);
  }

  /**
   * Shows `picture` with the layer's placement (hides the sprite when there is none); `scale` is
   * output pixels per sequence pixel. Returns the layer's box.
   */
  update(
    layer: Layer & { readonly box: Size },
    picture: Picture | undefined,
    frame: Size,
    scale: number,
  ): Size {
    place(this.root, layer, layer.box, frame);
    this.blur.apply(this.root, layer.transform.blur * scale);
    this.sprite.visible = picture !== undefined;
    if (picture === undefined) return layer.box;
    this.upload(picture.source);
    const crop = layer.kind === 'video' ? layer.crop : null;
    this.frameTexture(crop === null ? WHOLE : codedCrop(crop, picture.rotation, picture.mirrored));
    const fit = pictureFit(layer.box, picture.rotation, picture.mirrored);
    const texture = this.sprite.texture;
    this.sprite.position.set(fit.x, fit.y);
    this.sprite.rotation = fit.rotation;
    this.sprite.scale.set(
      (fit.width / texture.width) * (fit.mirrored ? -1 : 1),
      fit.height / texture.height,
    );
    return layer.box;
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
    this.sprite.texture = new Texture({
      source: this.source,
      frame: new Rectangle(0, 0, width, height),
      dynamic: true,
    });
  }

  /** Shows only `crop` (normalised in the decoded pixels) of the texture; unchanged crops cost nothing. */
  private frameTexture(crop: Crop): void {
    const source = this.source;
    if (source === null) return;
    const texture = this.sprite.texture;
    const x = crop.x * source.pixelWidth;
    const y = crop.y * source.pixelHeight;
    const width = Math.max(1, crop.width * source.pixelWidth);
    const height = Math.max(1, crop.height * source.pixelHeight);
    const frame = texture.frame;
    if (frame.x === x && frame.y === y && frame.width === width && frame.height === height) return;
    frame.x = x;
    frame.y = y;
    frame.width = width;
    frame.height = height;
    texture.orig.copyFrom(frame);
    texture.update();
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
    this.blur.destroy();
    this.root.destroy({ children: true });
  }
}
