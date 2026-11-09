import type { Picture } from '../composite/compositor';
import { FrameCache, type FrameCacheLimits } from '../decode/frame-cache';
import { pictureTracker } from '../decode/frame-tracker';
import { TEMPLATE_WATCHDOG_MS } from './contract';
import { errorPicture } from './error-picture';
import type {
  TemplateIssue,
  TemplateRenderRequest,
  TemplateRenderResult,
  TemplateRenderer,
} from './template-renderer';
import { TemplateFailure } from './template-worker';
import { TemplateWorkerPool, type WorkerPoolOptions } from './worker-pool';

/** Options of a {@link TemplateRuntime}. */
export interface TemplateRuntimeOptions extends WorkerPoolOptions {
  /** Longest a frame may take before its Worker is killed. */
  readonly watchdogMs?: number | undefined;
  readonly cacheLimits?: FrameCacheLimits | undefined;
}

/** Rendered frames kept: a few seconds of two templates at 30 fps, within 256 MiB. */
const DEFAULT_CACHE: FrameCacheLimits = { budgetBytes: 256 * 1024 * 1024, maxEntries: 96 };

/** Width of the time span a cached frame stands for (frames are a millisecond apart or more). */
const INSTANT = 1e-7;

/**
 * Renders motion template layers to pictures for the frame renderer (prompt 24, ADR-0035): one
 * sandboxed Worker per template version ({@link TemplateWorkerPool}), frames cached by template,
 * version, values, size and instant (leases, like decoded pictures), the fonts a template asks
 * for given to its Worker then the frame drawn again. A failing template gives the error picture
 * and a {@link TemplateIssue}; a frame overrunning the watchdog kills the Worker and marks the
 * version failed. Preview, export and frames for agents share this code (golden rule 7).
 */
export class TemplateRuntime implements TemplateRenderer {
  private readonly cache: FrameCache;
  private readonly pool: TemplateWorkerPool;
  private readonly issues = new Map<string, TemplateIssue>();
  private readonly inflight = new Map<string, Promise<void>>();
  private readonly watchdogMs: number;

  constructor(options: TemplateRuntimeOptions = {}) {
    this.pool = new TemplateWorkerPool(options);
    this.watchdogMs = options.watchdogMs ?? TEMPLATE_WATCHDOG_MS;
    this.cache = new FrameCache(options.cacheLimits ?? DEFAULT_CACHE, (picture) => {
      pictureTracker.close(picture.source);
    });
  }

  peek(request: TemplateRenderRequest): TemplateRenderResult | null {
    const { source, time, key } = cacheKey(request);
    const lease = this.cache.lease(source, time);
    return lease === null ? null : { lease, issue: this.issues.get(key) ?? null };
  }

  async render(request: TemplateRenderRequest): Promise<TemplateRenderResult> {
    const { key } = cacheKey(request);
    const running = this.inflight.get(key);
    if (running !== undefined) {
      await running;
      const cached = this.peek(request);
      if (cached !== null) return cached;
    }
    const work = this.produce(request);
    this.inflight.set(
      key,
      work.then(
        () => undefined,
        () => undefined,
      ),
    );
    try {
      return await work;
    } finally {
      this.inflight.delete(key);
    }
  }

  /** Cached frames and running Workers are dropped. */
  destroy(): void {
    this.pool.destroy();
    this.cache.clear();
    this.issues.clear();
  }

  /** Renders the frame (or the error picture) and caches it; returns a lease of it. */
  private async produce(request: TemplateRenderRequest): Promise<TemplateRenderResult> {
    const { source, time, key } = cacheKey(request);
    const outcome = await this.draw(request).then(
      (bitmap) => ({ bitmap, issue: null }),
      async (failure: unknown) => ({
        bitmap: await errorPicture(request.width, request.height),
        issue: issueOf(request, failure),
      }),
    );
    if (outcome.issue === null) this.issues.delete(key);
    else this.issues.set(key, outcome.issue);
    trimIssues(this.issues);
    const { bitmap } = outcome;
    const picture: Picture = {
      source: pictureTracker.adopt(bitmap, 'image'),
      rotation: 0,
      mirrored: false,
    };
    const bytes = bitmap.width * bitmap.height * 4;
    const lease = this.cache.insert(source, picture, time, time + INSTANT, bytes);
    return { lease, issue: outcome.issue };
  }

  private async draw(request: TemplateRenderRequest): Promise<ImageBitmap> {
    const { layer } = request;
    const entry = await this.pool.acquire(layer.template);
    if (entry.kind === 'failed') throw entry.failure;
    const { pooled } = entry;
    const params = await this.pool.inputs(pooled, layer.params, this.watchdogMs);
    const frameRequest = {
      time: layer.time,
      duration: layer.duration,
      width: request.width,
      height: request.height,
      designWidth: request.designWidth,
      designHeight: request.designHeight,
      params,
    };
    try {
      let frame = await pooled.worker.render(frameRequest, this.watchdogMs);
      if (frame.missingFamilies.length > 0) {
        const given = await this.pool.provideFonts(pooled, frame.missingFamilies, this.watchdogMs);
        if (given) {
          frame.bitmap.close();
          frame = await pooled.worker.render(frameRequest, this.watchdogMs);
        }
      }
      return frame.bitmap;
    } catch (failure) {
      if (failure instanceof TemplateFailure && failure.kind !== 'exception') {
        this.pool.markFailed(layer.template, failure);
      }
      throw failure;
    }
  }
}

/** Where a template frame is cached: its source id (template, version, size, values) and time. */
function cacheKey(request: TemplateRenderRequest): { source: string; time: number; key: string } {
  const { layer } = request;
  const source = [
    layer.template.id,
    String(layer.template.version),
    `${String(request.width)}x${String(request.height)}`,
    JSON.stringify(layer.params),
  ].join('|');
  return { source, time: layer.time, key: `${source}@${String(layer.time)}` };
}

/** Issues of frames long evicted are forgotten (the cache holds at most a few hundred frames). */
function trimIssues(issues: Map<string, TemplateIssue>): void {
  for (const key of issues.keys()) {
    if (issues.size <= 512) return;
    issues.delete(key);
  }
}

function issueOf(request: TemplateRenderRequest, failure: unknown): TemplateIssue {
  const place = failure instanceof TemplateFailure ? failure.places[0] : undefined;
  return {
    itemId: request.layer.itemId,
    templateId: request.layer.template.id,
    message: place?.message ?? (failure instanceof Error ? failure.message : String(failure)),
    line: place?.line ?? null,
  };
}
