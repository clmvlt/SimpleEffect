import type { Flicks, ItemId } from '@se/protocol';

import type { Compositor, Picture } from '../composite/compositor';
import { DecodeError } from '../decode/decode-error';
import type { DecoderPool, MediaFileRef } from '../decode/decoder-pool';
import type { PictureLease } from '../decode/frame-cache';
import { frameLookupTime } from '../decode/frame-time';
import { chooseVariant, type MediaLocator } from '../decode/media-variant';
import { evaluate, type EvaluableSequence } from '../evaluate/evaluate';
import { isPictureLayer, type FrameGraph, type PictureLayer } from '../evaluate/frame-graph';
import type { AssetTable } from '../evaluate/item-layer';
import { MediaIssues, type MediaIssue } from './media-issue';

/** The document a renderer draws: a sequence and the project's assets. */
export interface RenderScene {
  readonly sequence: EvaluableSequence;
  readonly assets: AssetTable;
}

/** One frame to draw. */
export interface FrameRequest {
  readonly scene: RenderScene;
  readonly time: Flicks;
  /** Prefer ready proxies (viewer option). */
  readonly proxy: boolean;
  /**
   * Drawn during playback: missing frames come from the sequential readers decoding ahead
   * (`DecoderPool.prefetch`) rather than from a random-access decode each.
   */
  readonly playback?: boolean | undefined;
}

/** What happened to a request. */
export type RenderOutcome =
  | { readonly kind: 'drawn'; readonly issues: readonly MediaIssue[] }
  /** A newer request replaced it before its pictures were ready: nothing was drawn. */
  | { readonly kind: 'superseded' };

/** How missing pictures are fetched. */
interface FetchMode {
  readonly signal: AbortSignal;
  readonly playback: boolean;
}

/** A picture layer whose pictures must be fetched, and the file to read. */
interface PictureNeed {
  readonly layer: PictureLayer;
  readonly file: MediaFileRef;
  /** For video layers: the media time (seconds) aimed at the shown frame. */
  readonly lookupTime: number;
}

/**
 * Draws frames: evaluate → pictures from the decoder pool → compositor (`docs/architecture.md`
 * §8). The same code serves the viewer and, later, export (golden rule 7).
 *
 * A request whose pictures are all cached is drawn synchronously. Otherwise `onLoading` fires, the
 * previous image stays on screen and the frame is drawn once every picture is decoded, unless a
 * newer request arrived meanwhile: then its pending decodes are cancelled and nothing is drawn.
 */
export class FrameRenderer {
  private current: AbortController | null = null;

  constructor(
    private readonly compositor: Compositor,
    private readonly pool: DecoderPool,
    private readonly locator: MediaLocator,
  ) {}

  /** Draws `request`; `onLoading` is called when it has to wait for decoding. */
  async render(
    request: FrameRequest,
    onLoading: () => void = () => undefined,
  ): Promise<RenderOutcome> {
    this.current?.abort();
    const controller = new AbortController();
    this.current = controller;
    const graph = evaluate(request.scene.sequence, request.scene.assets, request.time);
    const issues = new MediaIssues();
    const needs = this.pictureNeeds(graph, request, issues);
    const leases = new Map<ItemId, PictureLease>();
    const missing: PictureNeed[] = [];
    for (const need of needs) {
      const cached = this.peek(need);
      if (cached === null) missing.push(need);
      else leases.set(need.layer.itemId, cached);
    }
    if (missing.length > 0) {
      onLoading();
      await this.fetchAll(missing, leases, issues, {
        signal: controller.signal,
        playback: request.playback === true,
      });
      if (controller.signal.aborted) {
        releaseAll(leases);
        return { kind: 'superseded' };
      }
    }
    try {
      const pictures = new Map<ItemId, Picture>();
      for (const [itemId, lease] of leases) pictures.set(itemId, lease.picture);
      this.compositor.render(graph, pictures);
    } finally {
      releaseAll(leases);
    }
    if (this.current === controller) this.current = null;
    return { kind: 'drawn', issues: issues.list() };
  }

  /** Cancels the pending request, if any. */
  cancel(): void {
    this.current?.abort();
    this.current = null;
  }

  /** The files to read for the picture layers; layers without a playable file become issues. */
  private pictureNeeds(
    graph: FrameGraph,
    request: FrameRequest,
    issues: MediaIssues,
  ): PictureNeed[] {
    const needs: PictureNeed[] = [];
    for (const layer of graph.layers) {
      if (!isPictureLayer(layer)) continue;
      const asset = request.scene.assets[layer.assetId];
      if (asset === undefined) continue;
      const choice = chooseVariant(asset, { proxy: request.proxy });
      if (choice.kind === 'preparing') {
        issues.add({ assetId: layer.assetId, kind: 'preparing' });
        continue;
      }
      if (choice.kind === 'unavailable') {
        issues.add({ assetId: layer.assetId, kind: choice.reason });
        continue;
      }
      const url = this.locator(layer.assetId, choice.variant);
      if (url === null) {
        issues.add({ assetId: layer.assetId, kind: 'preparing' });
        continue;
      }
      const frameRate = asset.mediaInfo?.videoStream?.frameRate ?? null;
      const lookupTime = layer.kind === 'video' ? frameLookupTime(layer.sourceTime, frameRate) : 0;
      needs.push({
        layer,
        file: { assetId: layer.assetId, variant: choice.variant, url },
        lookupTime,
      });
    }
    return needs;
  }

  private peek(need: PictureNeed): PictureLease | null {
    return need.layer.kind === 'video'
      ? this.pool.peekVideoFrame(need.file, need.lookupTime)
      : this.pool.peekImage(need.file);
  }

  private fetch(need: PictureNeed, mode: FetchMode): Promise<PictureLease | null> {
    if (need.layer.kind !== 'video') return this.pool.image(need.file);
    return mode.playback
      ? this.pool.playbackFrame(need.file, need.layer.itemId, need.lookupTime, mode.signal)
      : this.pool.videoFrame(need.file, need.lookupTime, mode.signal);
  }

  private async fetchAll(
    missing: readonly PictureNeed[],
    leases: Map<ItemId, PictureLease>,
    issues: MediaIssues,
    mode: FetchMode,
  ): Promise<void> {
    const results = await Promise.allSettled(missing.map((need) => this.fetch(need, mode)));
    results.forEach((result, index) => {
      const need = missing[index];
      if (need === undefined) return;
      if (result.status === 'fulfilled') {
        if (result.value !== null) leases.set(need.layer.itemId, result.value);
        return;
      }
      const error: unknown = result.reason;
      issues.add({
        assetId: need.file.assetId,
        kind: 'decodeFailed',
        ...(error instanceof DecodeError ? { failure: error.failure } : {}),
        message: error instanceof Error ? error.message : String(error),
      });
    });
  }
}

function releaseAll(leases: ReadonlyMap<ItemId, PictureLease>): void {
  for (const lease of leases.values()) lease.release();
}
