import type { ItemId } from '@se/protocol';

import type { Compositor, Picture, PictureSet } from '../composite/compositor';
import type { FrameGraph } from '../evaluate/frame-graph';

/** A compositor for Node tests: records what it was asked to draw (no GPU). */
export class RecordingCompositor implements Compositor {
  readonly draws: { graph: FrameGraph; pictures: Map<ItemId, Picture> }[] = [];
  destroyed = false;
  resize(): void {}
  setBackground(): void {}
  render(graph: FrameGraph, pictures: PictureSet): void {
    this.draws.push({ graph, pictures: new Map(pictures) });
  }
  destroy(): void {
    this.destroyed = true;
  }
}
