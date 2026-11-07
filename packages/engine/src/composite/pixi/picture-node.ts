import { Container, ImageSource, Sprite, Texture } from 'pixi.js';

import type { Layer, Size } from '../../evaluate/frame-graph';
import type { Picture } from '../compositor';
import { pictureFit } from '../layout';
import { LayerBlur, place, type LayerNode } from './layer-node';

/** Pixel size Pixi reads from a picture (`displayWidth` for frames, `width` for bitmaps). */
function resourceSize(resource: VideoFrame | ImageBitmap): readonly [number, number] {
  return 'displayWidth' in resource
    ? [resource.displayWidth, resource.displayHeight]
    : [resource.width, resource.height];
}

/** A video frame or an image stretched over the layer's box. */
export class PictureNode implements LayerNode {
  readonly kind;
  readonly root = new Container();
  private readonly sprite = new Sprite();
  private readonly blur = new LayerBlur();
  private source: ImageSource | null = null;

  constructor(kind: 'video' | 'image') {
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
    this.blur.destroy();
    this.root.destroy({ children: true });
  }
}
