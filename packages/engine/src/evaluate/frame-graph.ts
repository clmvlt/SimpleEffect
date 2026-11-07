import type { AssetId, Flicks, ItemId, Vec2 } from '@se/protocol';

import type { Rgba } from '../animation/color';

/** A width and a height in pixels of the sequence frame. */
export interface Size {
  readonly width: number;
  readonly height: number;
}

/**
 * A transform with every keyframe resolved at the evaluated time, in the domain's units
 * (ADR-0015): `position` normalised in the sequence frame, `anchor` normalised in the layer's box,
 * relative `scale`, clockwise `rotation` in degrees, `opacity` in `[0, 1]` (fades included).
 */
export interface ResolvedTransform {
  readonly position: Vec2;
  readonly scale: Vec2;
  readonly rotation: number;
  readonly anchor: Vec2;
  readonly opacity: number;
}

interface LayerBase {
  /** The item drawn by this layer. */
  readonly itemId: ItemId;
  readonly transform: ResolvedTransform;
}

/** A frame of a video asset. */
export interface VideoLayer extends LayerBase {
  readonly kind: 'video';
  readonly assetId: AssetId;
  /** Time in the asset whose frame is shown (`0` = its first frame). */
  readonly sourceTime: Flicks;
  /**
   * The picture's natural box in sequence pixels: the asset's display size fitted or filled into
   * the frame (`reframe`). It does not depend on the decoded variant, so proxies keep the framing.
   */
  readonly box: Size;
}

/** A still image asset, fitted into the frame at scale 1. */
export interface ImageLayer extends LayerBase {
  readonly kind: 'image';
  readonly assetId: AssetId;
  readonly box: Size;
}

/** A filled rectangle or ellipse. */
export interface ShapeLayer extends LayerBase {
  readonly kind: 'shape';
  readonly geometry: 'rectangle' | 'ellipse';
  /** Box in sequence pixels (design pixels × design scale). */
  readonly box: Size;
  /** Corner radius in sequence pixels (rectangles; 0 for ellipses). */
  readonly cornerRadius: number;
  readonly fill: Rgba;
}

/** A block of text, measured by the compositor (its box depends on font metrics). */
export interface TextLayer extends LayerBase {
  readonly kind: 'text';
  readonly content: string;
  readonly fontFamily: string;
  /** Font size in sequence pixels (design pixels × design scale). */
  readonly fontSize: number;
  readonly color: Rgba;
}

/** Something to draw, in stacking order within {@link FrameGraph.layers}. */
export type Layer = VideoLayer | ImageLayer | ShapeLayer | TextLayer;

/** A layer that needs decoded pixels (a video frame or an image). */
export type PictureLayer = VideoLayer | ImageLayer;

/**
 * Everything visible in a sequence at one instant, bottom layer first: the pure output of
 * `evaluate`, the input of a `Compositor`. Sizes are in pixels of the sequence frame.
 */
export interface FrameGraph {
  /** The evaluated sequence time. */
  readonly time: Flicks;
  /** The sequence frame size in pixels. */
  readonly frame: Size;
  /** Real pixels per design pixel: shorter side / 1080 (ADR-0015). */
  readonly designScale: number;
  readonly layers: readonly Layer[];
}

/** Whether a layer needs decoded pixels. */
export function isPictureLayer(layer: Layer): layer is PictureLayer {
  return layer.kind === 'video' || layer.kind === 'image';
}
