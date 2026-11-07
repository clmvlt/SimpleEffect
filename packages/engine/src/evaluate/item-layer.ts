import type { Asset, AssetId, Flicks, Item, ItemKind, Transform2D } from '@se/protocol';

import { BLACK, parseColor } from '../animation/color';
import type { Layer, Size } from './frame-graph';
import { assetDisplaySize, reframedBox, shapeBox } from './geometry';
import { clipSourceTime, fadeFactor } from './timing';
import { isInvisible, resolveTransform } from './transform';

/** Assets of the project by id (the renderer mirror's shape). */
export type AssetTable = Readonly<Record<AssetId, Asset>>;

/** What a layer is built from: the item, the evaluated time, the frame and the assets. */
export interface LayerContext {
  readonly item: Pick<Item, 'id' | 'timelineRange'>;
  readonly time: Flicks;
  readonly frame: Size;
  readonly designScale: number;
  readonly assets: AssetTable;
}

/**
 * Fixed look of captions until caption styles exist.
 * TODO(prompt-23): caption styles (and word highlighting) replace this.
 */
const CAPTION_LOOK = {
  fontFamily: 'Inter',
  fontSize: 56,
  color: { r: 1, g: 1, b: 1, a: 1 },
  transform: {
    position: { kind: 'static', value: { x: 0.5, y: 0.9 } },
    scale: { kind: 'static', value: { x: 1, y: 1 } },
    rotation: { kind: 'static', value: 0 },
    anchor: { kind: 'static', value: { x: 0.5, y: 1 } },
    opacity: { kind: 'static', value: 1 },
  } satisfies Transform2D,
} as const;

function mediaClipLayer(
  clip: Extract<ItemKind, { kind: 'mediaClip' }>,
  context: LayerContext,
): Layer | null {
  const asset = context.assets[clip.assetId];
  if (asset?.kind !== 'video') return null;
  const range = context.item.timelineRange;
  const localTime = (context.time - range.start) as Flicks;
  const fade = fadeFactor(clip, range.duration, localTime);
  return {
    kind: 'video',
    itemId: context.item.id,
    assetId: clip.assetId,
    sourceTime: clipSourceTime(clip, range, context.time),
    box: reframedBox(assetDisplaySize(asset), context.frame, clip.reframe.kind),
    transform: resolveTransform(clip.transform, localTime, fade),
  };
}

/** Builds the layer of one kind of item at the context's time; `null` when it draws nothing. */
function layerOfKind(kind: ItemKind, context: LayerContext, localTime: Flicks): Layer | null {
  const base = { itemId: context.item.id };
  switch (kind.kind) {
    case 'mediaClip':
      return mediaClipLayer(kind, context);
    case 'image': {
      const asset = context.assets[kind.assetId];
      if (asset?.kind !== 'image') return null;
      const box = reframedBox(assetDisplaySize(asset), context.frame, 'fit');
      const transform = resolveTransform(kind.transform, localTime);
      return { ...base, kind: 'image', assetId: kind.assetId, box, transform };
    }
    case 'shape': {
      const { box, cornerRadius } = shapeBox(kind.geometry, context.designScale);
      const fill = parseColor(kind.fill) ?? BLACK;
      const transform = resolveTransform(kind.transform, localTime);
      return {
        ...base,
        kind: 'shape',
        geometry: kind.geometry.kind,
        box,
        cornerRadius,
        fill,
        transform,
      };
    }
    case 'text':
      return {
        ...base,
        kind: 'text',
        content: kind.content,
        fontFamily: kind.style.fontFamily,
        fontSize: kind.style.fontSize * context.designScale,
        color: parseColor(kind.style.color) ?? BLACK,
        transform: resolveTransform(kind.transform, localTime),
      };
    case 'caption':
      return {
        ...base,
        kind: 'text',
        content: kind.text,
        fontFamily: CAPTION_LOOK.fontFamily,
        fontSize: CAPTION_LOOK.fontSize * context.designScale,
        color: CAPTION_LOOK.color,
        transform: resolveTransform(CAPTION_LOOK.transform, localTime),
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
