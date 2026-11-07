import type { Asset, AssetId, Flicks, Item, ItemKind, TextItem, Transform2D } from '@se/protocol';

import type { Layer, Size } from './frame-graph';
import { interpolateVec2, valueAt } from '../animation/interpolate';
import { assetDisplaySize, imageBox } from './geometry';
import { shapeLayerStyle, textLayerStyle, type GraphicContext } from './graphic-layers';
import { CENTRE_FOCUS, reframedPicture } from './reframe';
import { clipSourceTime, fadeFactor } from './timing';
import { isInvisible, resolveTransform } from './transform';

/** Assets of the project by id (the renderer mirror's shape). */
export type AssetTable = Readonly<Record<AssetId, Asset>>;

/**
 * What a layer is built from: the item, the evaluated time (outside the item's range when it plays
 * through a transition), the frame and the assets.
 */
export interface LayerContext {
  readonly item: Pick<Item, 'id' | 'timelineRange'>;
  readonly time: Flicks;
  readonly frame: Size;
  readonly designScale: number;
  readonly assets: AssetTable;
  /** Extra opacity factor (a dissolve's weight); 1 when absent. */
  readonly opacity?: number;
}

/** Widest caption block in design pixels (narrower frames get 90 % of their width). */
const CAPTION_MAX_WIDTH = 1600;

/**
 * Fixed look of captions until caption styles exist: white Inter 56 px, bottom centre.
 * TODO(prompt-23): caption styles (and word highlighting) replace this.
 */
const CAPTION_LOOK: Omit<TextItem, 'content'> = {
  style: {
    fontFamily: 'Inter',
    fontWeight: 600,
    italic: false,
    fontSize: { kind: 'static', value: 56 },
    color: { kind: 'static', value: '#ffffff' },
    stroke: {
      enabled: false,
      color: { kind: 'static', value: '#000000' },
      width: { kind: 'static', value: 0 },
    },
    shadow: {
      enabled: true,
      color: { kind: 'static', value: '#000000b3' },
      blur: { kind: 'static', value: 6 },
      offset: { kind: 'static', value: { x: 0, y: 2 } },
    },
    background: {
      enabled: false,
      color: { kind: 'static', value: '#000000b3' },
      padding: { x: 0, y: 0 },
      cornerRadius: 0,
    },
    letterSpacing: { kind: 'static', value: 0 },
    lineHeight: 1.2,
    align: 'center',
    maxWidth: CAPTION_MAX_WIDTH,
    textCase: 'normal',
  },
  transform: {
    position: { kind: 'static', value: { x: 0.5, y: 0.9 } },
    scale: { kind: 'static', value: { x: 1, y: 1 } },
    rotation: { kind: 'static', value: 0 },
    anchor: { kind: 'static', value: { x: 0.5, y: 1 } },
    opacity: { kind: 'static', value: 1 },
    blur: { kind: 'static', value: 0 },
  } satisfies Transform2D,
  reveal: { kind: 'static', value: 1 },
};

/** The caption look for `content`, no wider than 90 % of the frame (portrait formats). */
function captionLook(content: string, context: LayerContext): TextItem {
  const frameWidth = context.frame.width / context.designScale;
  const maxWidth = Math.min(CAPTION_MAX_WIDTH, Math.floor(frameWidth * 0.9));
  return { ...CAPTION_LOOK, content, style: { ...CAPTION_LOOK.style, maxWidth } };
}

function mediaClipLayer(
  clip: Extract<ItemKind, { kind: 'mediaClip' }>,
  context: LayerContext,
): Layer | null {
  const asset = context.assets[clip.assetId];
  if (asset?.kind !== 'video') return null;
  const range = context.item.timelineRange;
  const localTime = (context.time - range.start) as Flicks;
  const fade = fadeFactor(clip, range.duration, localTime) * (context.opacity ?? 1);
  const { reframe } = clip;
  const focus =
    reframe.kind === 'fill' || reframe.kind === 'tracked'
      ? valueAt(reframe.focus, localTime, interpolateVec2, CENTRE_FOCUS)
      : CENTRE_FOCUS;
  const picture = reframedPicture(
    assetDisplaySize(asset),
    context.frame,
    reframe.kind,
    focus,
    context.designScale,
  );
  return {
    kind: 'video',
    itemId: context.item.id,
    assetId: clip.assetId,
    sourceTime: clipSourceTime(clip, range, context.time),
    box: picture.box,
    crop: picture.crop,
    transform: resolveTransform(clip.transform, localTime, fade, context.designScale),
  };
}

/** Builds the layer of one kind of item at the context's time; `null` when it draws nothing. */
function layerOfKind(kind: ItemKind, context: LayerContext, localTime: Flicks): Layer | null {
  const itemId = context.item.id;
  const weight = context.opacity ?? 1;
  const graphic: GraphicContext = {
    localTime,
    frame: context.frame,
    designScale: context.designScale,
  };
  const placed = (transform: Transform2D) =>
    resolveTransform(transform, localTime, weight, context.designScale);
  switch (kind.kind) {
    case 'mediaClip':
      return mediaClipLayer(kind, context);
    case 'image': {
      const asset = context.assets[kind.assetId];
      if (asset?.kind !== 'image') return null;
      const box = imageBox(assetDisplaySize(asset), context.frame, kind.fit, context.designScale);
      return {
        itemId,
        kind: 'image',
        assetId: kind.assetId,
        box,
        transform: placed(kind.transform),
      };
    }
    case 'shape':
      return { itemId, ...shapeLayerStyle(kind, graphic), transform: placed(kind.transform) };
    case 'text':
      return { itemId, ...textLayerStyle(kind, graphic), transform: placed(kind.transform) };
    case 'caption':
      return {
        itemId,
        ...textLayerStyle(captionLook(kind.text, context), graphic),
        transform: placed(CAPTION_LOOK.transform),
      };
    case 'motionTemplate':
      // TODO(prompt-24): motion templates render through the template runtime.
      return null;
  }
}

/**
 * The layer drawing `kind` at the context's time, or `null` when nothing is visible: audio-only
 * clips, unknown assets, templates (not rendered yet), invisible transforms.
 */
export function itemLayer(kind: ItemKind, context: LayerContext): Layer | null {
  const localTime = (context.time - context.item.timelineRange.start) as Flicks;
  const layer = layerOfKind(kind, context, localTime);
  return layer === null || isInvisible(layer.transform) ? null : layer;
}
