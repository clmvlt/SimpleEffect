import type {
  Asset,
  AssetId,
  CaptionStyle,
  Flicks,
  Item,
  ItemKind,
  Transform2D,
} from '@se/protocol';

import type { Layer, Size } from './frame-graph';
import { interpolateVec2, valueAt } from '../animation/interpolate';
import { DEFAULT_CAPTION_PRESET, presetStyle } from '../caption-styles';
import { captionLayerStyle, captionTransform } from './caption-layer';
import { assetDisplaySize, imageBox } from './geometry';
import { shapeLayerStyle, textLayerStyle, type GraphicContext } from './graphic-layers';
import { CENTRE_FOCUS, reframedPicture } from './reframe';
import { clipSourceTime, fadeFactor } from './timing';
import { isInvisible, resolveTransform } from './transform';

/** Assets of the project by id (the renderer mirror's shape). */
export type AssetTable = Readonly<Record<AssetId, Asset>>;

/** The project's caption styles (prompt 23). */
export type CaptionStyleList = readonly CaptionStyle[];

/** Draws captions whose style is missing (never in a valid project; old scenes). */
const FALLBACK_CAPTION_STYLE = presetStyle(DEFAULT_CAPTION_PRESET, '');

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
  /** The project's caption styles (captions are drawn with the default preset without them). */
  readonly captionStyles?: CaptionStyleList | undefined;
  /** Extra opacity factor (a dissolve's weight); 1 when absent. */
  readonly opacity?: number;
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
    case 'caption': {
      const style =
        context.captionStyles?.find((candidate) => candidate.id === kind.styleId) ??
        FALLBACK_CAPTION_STYLE;
      const caption = captionLayerStyle(kind, style, graphic);
      return caption === null
        ? null
        : { itemId, ...caption, transform: captionTransform(style, weight) };
    }
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
