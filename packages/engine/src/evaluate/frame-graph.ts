import type { AssetId, Flicks, ItemId, TextAlign, TransitionId, Vec2 } from '@se/protocol';

import type { Rgba } from '../animation/color';

/** A width and a height in pixels of the sequence frame. */
export interface Size {
  readonly width: number;
  readonly height: number;
}

/** A rectangle normalised in an upright picture (`(0, 0)` top-left, `(1, 1)` bottom-right). */
export interface Crop {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/**
 * A transform with every keyframe resolved at the evaluated time, in the domain's units
 * (ADR-0015): `position` normalised in the sequence frame, `anchor` normalised in the layer's box,
 * relative `scale`, clockwise `rotation` in degrees, `opacity` in `[0, 1]` (fades included),
 * `blur` in sequence pixels (design pixels × design scale; 0 = sharp).
 */
export interface ResolvedTransform {
  readonly position: Vec2;
  readonly scale: Vec2;
  readonly rotation: number;
  readonly anchor: Vec2;
  readonly opacity: number;
  readonly blur: number;
}

/** An outline: its colour and visible width outside the edge, in sequence pixels. */
export interface ResolvedStroke {
  readonly color: Rgba;
  readonly width: number;
}

/**
 * A layer's part in a cross dissolve: the outgoing and incoming layers of one transition are
 * listed one after the other and composited as one group (`DISSOLVE_COMPOSITING`), their opacities
 * already weighted by the dissolve's progress (`1 − p` and `p`).
 */
export interface DissolveRole {
  readonly transitionId: TransitionId;
  readonly side: 'outgoing' | 'incoming';
}

/**
 * How a compositor draws a dissolve group: alone in an empty layer (isolated), the outgoing layer
 * normally and the incoming one added (premultiplied `src + dst`), then the group over what lies
 * below. With weights `1 − p` and `p` this gives exactly `(1 − p)·(A over below) + p·(B over
 * below)`: no dip to the lower tracks halfway.
 */
export const DISSOLVE_COMPOSITING = 'isolated-additive' as const;

interface LayerBase {
  /** The item drawn by this layer. */
  readonly itemId: ItemId;
  readonly transform: ResolvedTransform;
  /** Set when the layer is a side of a cross dissolve. */
  readonly dissolve?: DissolveRole;
}

/** A frame of a video asset. */
export interface VideoLayer extends LayerBase {
  readonly kind: 'video';
  readonly assetId: AssetId;
  /** Time in the asset whose frame is shown (`0` = its first frame). */
  readonly sourceTime: Flicks;
  /**
   * The picture's natural box in sequence pixels, from the asset's display size and the clip's
   * reframe mode (`reframedPicture`): fitted into the frame, the frame itself when filling, or
   * the natural size. It does not depend on the decoded variant, so proxies keep the framing.
   */
  readonly box: Size;
  /** The part of the upright picture stretched over the box (fill modes); `null` = all of it. */
  readonly crop: Crop | null;
}

/** A still image asset, fitted into the frame at scale 1. */
export interface ImageLayer extends LayerBase {
  readonly kind: 'image';
  readonly assetId: AssetId;
  readonly box: Size;
}

/** A filled rectangle, ellipse or line (a capsule of the line's thickness), maybe outlined. */
export interface ShapeLayer extends LayerBase {
  readonly kind: 'shape';
  readonly geometry: 'rectangle' | 'ellipse' | 'line';
  /** Box in sequence pixels (design pixels × design scale; the frame for covering shapes). */
  readonly box: Size;
  /** Corner radius in sequence pixels (rectangles; half the thickness for lines; else 0). */
  readonly cornerRadius: number;
  readonly fill: Rgba;
  readonly stroke: ResolvedStroke | null;
}

/** The font a text asks for (CSS matching picks the face). */
export interface FontRequest {
  readonly family: string;
  /** 1 to 1000. */
  readonly weight: number;
  readonly italic: boolean;
}

/** A drop shadow, in sequence pixels. */
export interface ResolvedShadow {
  readonly color: Rgba;
  readonly blur: number;
  readonly offset: Vec2;
}

/** A plate behind a text block, in sequence pixels. */
export interface ResolvedBackground {
  readonly color: Rgba;
  readonly padding: Vec2;
  readonly cornerRadius: number;
}

/**
 * A block of text, laid out by the compositor (its box depends on font metrics). Every size is in
 * sequence pixels (design pixels × design scale).
 */
export interface TextLayer extends LayerBase {
  readonly kind: 'text';
  /** The text to lay out, letter case applied. */
  readonly content: string;
  readonly font: FontRequest;
  readonly fontSize: number;
  readonly color: Rgba;
  readonly letterSpacing: number;
  /** Distance between baselines as a multiple of `fontSize`. */
  readonly lineHeight: number;
  readonly align: TextAlign;
  /** Wrapping width; `null` keeps each line on one line. */
  readonly maxWidth: number | null;
  readonly stroke: ResolvedStroke | null;
  readonly shadow: ResolvedShadow | null;
  /** When set, the layer's box includes the plate. */
  readonly background: ResolvedBackground | null;
  /** Share of the characters shown (0 to 1), laid out as the whole text. */
  readonly reveal: number;
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
