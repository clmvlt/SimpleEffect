import type { Flicks, ItemId } from '@se/protocol';

import type { LayerBoxes } from '../composite/compositor';
import type {
  Crop,
  FontRequest,
  FrameGraph,
  ResolvedTransform,
  Size,
} from '../evaluate/frame-graph';

/** Where one drawn layer sits: its resolved transform and its box in sequence pixels. */
export interface LayerLayout {
  readonly itemId: ItemId;
  readonly transform: ResolvedTransform;
  readonly box: Size;
  /** The part of the picture a video layer shows (fill modes), else `null`. */
  readonly crop: Crop | null;
}

/**
 * The layers of a drawn frame, bottom first, with what places them: the viewer's transform
 * handles and hit tests read it (texts are only measurable while drawing).
 */
export interface FrameLayout {
  readonly time: Flicks;
  readonly frame: Size;
  readonly layers: readonly LayerLayout[];
}

/** The layout of `graph` drawn with `boxes`. Dissolve sides keep their own entry. */
export function frameLayout(graph: FrameGraph, boxes: LayerBoxes): FrameLayout {
  const layers: LayerLayout[] = [];
  for (const layer of graph.layers) {
    const box = boxes.get(layer.itemId);
    if (box === undefined) continue;
    const crop = layer.kind === 'video' ? layer.crop : null;
    layers.push({ itemId: layer.itemId, transform: layer.transform, box, crop });
  }
  return { time: graph.time, frame: graph.frame, layers };
}

/** The fonts the text layers of `graph` ask for, each once. */
export function frameFonts(graph: FrameGraph): FontRequest[] {
  const fonts = new Map<string, FontRequest>();
  for (const layer of graph.layers) {
    if (layer.kind !== 'text') continue;
    const { family, weight, italic } = layer.font;
    fonts.set(`${family}|${String(weight)}|${String(italic)}`, layer.font);
  }
  return [...fonts.values()];
}
