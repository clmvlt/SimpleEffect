import type { ItemId } from '@se/protocol';

import type { PictureLease } from '../decode/frame-cache';
import type { FrameGraph, Size } from '../evaluate/frame-graph';
import type {
  TemplateIssue,
  TemplateRenderRequest,
  TemplateRenderer,
} from '../templates/template-renderer';

/**
 * The motion template layers of one frame (prompt 24): their frames leased from the template
 * runtime when cached, rendered otherwise, at the compositor's output size (at most twice the
 * sequence frame), each with its issue when the template failed.
 */
export class TemplateFrames {
  readonly issues: TemplateIssue[] = [];
  private readonly requests: TemplateRenderRequest[] = [];
  private missing: TemplateRenderRequest[] = [];

  constructor(
    private readonly renderer: TemplateRenderer | null,
    graph: FrameGraph,
    output: Size,
  ) {
    if (renderer === null) return;
    const scale = Math.min(2, output.width / graph.frame.width, output.height / graph.frame.height);
    for (const layer of graph.layers) {
      if (layer.kind !== 'template') continue;
      this.requests.push({
        layer,
        width: Math.max(2, Math.round(graph.frame.width * scale)),
        height: Math.max(2, Math.round(graph.frame.height * scale)),
        designWidth: graph.frame.width / graph.designScale,
        designHeight: graph.frame.height / graph.designScale,
      });
    }
  }

  /** Whether some template frames must still be rendered. */
  get waiting(): boolean {
    return this.missing.length > 0;
  }

  /** Leases the cached frames; remembers the others. */
  peek(leases: Map<ItemId, PictureLease>): void {
    if (this.renderer === null) return;
    for (const request of this.requests) {
      const cached = this.renderer.peek(request);
      if (cached === null) this.missing.push(request);
      else this.keep(leases, request, cached.lease, cached.issue);
    }
  }

  /** Renders the frames not cached (never rejects: failures come as error pictures). */
  async render(leases: Map<ItemId, PictureLease>): Promise<void> {
    const renderer = this.renderer;
    if (renderer === null) return;
    const missing = this.missing;
    this.missing = [];
    const results = await Promise.all(missing.map((request) => renderer.render(request)));
    results.forEach((result, index) => {
      const request = missing[index];
      if (request === undefined) result.lease.release();
      else this.keep(leases, request, result.lease, result.issue);
    });
  }

  private keep(
    leases: Map<ItemId, PictureLease>,
    request: TemplateRenderRequest,
    lease: PictureLease,
    issue: TemplateIssue | null,
  ): void {
    leases.get(request.layer.itemId)?.release();
    leases.set(request.layer.itemId, lease);
    if (issue !== null) this.issues.push({ ...issue, itemId: request.layer.itemId });
  }
}
