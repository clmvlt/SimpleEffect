import type { ItemId } from '@se/protocol';

import type { Compositor, LayerBoxes, Picture, PictureSet } from '../composite/compositor';
import type { FrameGraph, Size } from '../evaluate/frame-graph';

/**
 * A compositor for Node tests: records what it was asked to draw (no GPU). Text boxes are
 * estimated (0.6 em per character, one line per `\n`): there is no font to measure.
 */
export class RecordingCompositor implements Compositor {
  readonly draws: { graph: FrameGraph; pictures: Map<ItemId, Picture> }[] = [];
  destroyed = false;
  resize(): void {}
  setBackground(): void {}
  render(graph: FrameGraph, pictures: PictureSet): LayerBoxes {
    this.draws.push({ graph, pictures: new Map(pictures) });
    const boxes = new Map<ItemId, Size>();
    for (const layer of graph.layers) {
      if (layer.kind !== 'text') {
        boxes.set(layer.itemId, layer.box);
        continue;
      }
      const lines = layer.content.split('\n');
      const longest = Math.max(...lines.map((line) => line.length));
      boxes.set(layer.itemId, {
        width: longest * layer.fontSize * 0.6,
        height: lines.length * layer.fontSize * layer.lineHeight,
      });
    }
    return boxes;
  }
  destroy(): void {
    this.destroyed = true;
  }
}
