import type { AssetId } from '@se/protocol';

import type { TemplateValues, TemplateRef } from '../evaluate/template-layer';
import { compileTemplate } from './compile';
import { TEMPLATE_LOAD_WATCHDOG_MS, type TemplateParamInput } from './contract';
import type { TemplateFontSource } from './font-faces';
import { templateWorkerSource } from './sandbox-source';
import {
  TemplateFailure,
  TemplateWorker,
  blobWorkerFactory,
  type WorkerFactory,
} from './template-worker';

/** Gives template Workers the pictures of image assets (a bitmap the caller may transfer). */
export type TemplateImageSource = (assetId: AssetId) => Promise<ImageBitmap | null>;

/** The family templates draw with by default (bundled with the app). */
export const DEFAULT_TEMPLATE_FAMILY = 'Inter';

/** A running Worker with what it was given. */
export interface PooledWorker {
  readonly worker: TemplateWorker;
  /** Families asked for once (lower case), given or not. */
  readonly families: Set<string>;
  /** Images given, with their natural size. */
  readonly images: Map<AssetId, { readonly width: number; readonly height: number }>;
}

/** A template version's Worker, or why it cannot run. */
export type PoolEntry =
  | { readonly kind: 'ready'; readonly pooled: PooledWorker }
  | { readonly kind: 'failed'; readonly failure: TemplateFailure };

/** Options of a {@link TemplateWorkerPool}. */
export interface WorkerPoolOptions {
  readonly fonts?: TemplateFontSource | undefined;
  readonly images?: TemplateImageSource | undefined;
  readonly createWorker?: WorkerFactory | undefined;
  /** Workers kept running at once (least recently used ones stop). */
  readonly maxWorkers?: number | undefined;
  readonly loadWatchdogMs?: number | undefined;
}

/**
 * One Worker per template version (compiled and loaded once), least recently used ones stopped
 * beyond `maxWorkers`. A version that does not compile, does not load or overran its watchdog is
 * remembered as failed: frames of it show the error picture at once instead of stalling.
 */
export class TemplateWorkerPool {
  private readonly entries = new Map<string, Promise<PoolEntry>>();
  /** Families each template version turned out to draw with (lower case), given at start. */
  private readonly familiesOf = new Map<string, Set<string>>();
  private readonly maxWorkers: number;

  constructor(private readonly options: WorkerPoolOptions = {}) {
    this.maxWorkers = options.maxWorkers ?? 8;
  }

  /**
   * The Worker of `template` (started the first time and given the default font and the
   * families the version is known to draw with).
   */
  acquire(template: TemplateRef): Promise<PoolEntry> {
    const key = templateKey(template);
    const existing = this.entries.get(key);
    if (existing !== undefined) {
      this.entries.delete(key);
      this.entries.set(key, existing);
      return existing;
    }
    const entry = this.start(template);
    this.entries.set(key, entry);
    this.trim();
    return entry;
  }

  /** Remembers that `template` overran or crashed: its frames fail at once from now on. */
  markFailed(template: TemplateRef, failure: TemplateFailure): void {
    const key = templateKey(template);
    void this.entries.get(key)?.then((entry) => {
      if (entry.kind === 'ready') entry.pooled.worker.terminate();
    });
    this.entries.set(key, Promise.resolve({ kind: 'failed', failure }));
  }

  /** `values` as `render` receives them, after giving the Worker the images they name. */
  async inputs(
    pooled: PooledWorker,
    values: TemplateValues,
    watchdogMs: number,
  ): Promise<Record<string, TemplateParamInput>> {
    const wanted = new Set<AssetId>();
    for (const value of Object.values(values)) {
      if (typeof value === 'object' && value !== null && !pooled.images.has(value.assetId)) {
        wanted.add(value.assetId);
      }
    }
    if (wanted.size > 0) await this.sendImages(pooled, [...wanted], watchdogMs);
    const inputs: Record<string, TemplateParamInput> = {};
    for (const [name, value] of Object.entries(values)) {
      if (typeof value !== 'object' || value === null) {
        inputs[name] = value;
        continue;
      }
      const size = pooled.images.get(value.assetId);
      inputs[name] = size === undefined ? null : { kind: 'image', id: value.assetId, ...size };
    }
    return inputs;
  }

  /** Gives the Worker the faces of `families` not asked for yet; whether any face was sent. */
  async provideFonts(
    pooled: PooledWorker,
    families: readonly string[],
    watchdogMs: number,
  ): Promise<boolean> {
    const source = this.options.fonts;
    const fresh = families.filter((family) => !pooled.families.has(family.toLowerCase()));
    for (const family of fresh) pooled.families.add(family.toLowerCase());
    if (source === undefined || fresh.length === 0) return false;
    const faces = (await Promise.all(fresh.map((family) => source(family)))).flat();
    if (faces.length === 0) return false;
    await pooled.worker.sendFonts(faces, watchdogMs);
    return true;
  }

  /**
   * After `pooled` drew a frame with families it lacked: a Worker whose text used a family
   * before its faces arrived keeps the fallback font even once they load (Chromium resolves a
   * family once per Worker), so the version's Worker is replaced by one given those families
   * first (the old one retires after its queued frames). Resolves with the Worker to draw again
   * with — this one, or the one another frame's restart already made —, or `null` when the
   * catalogue has none of the families (the fallback is then the right picture).
   */
  async restartWithFonts(
    template: TemplateRef,
    pooled: PooledWorker,
    families: readonly string[],
  ): Promise<PoolEntry | null> {
    const key = templateKey(template);
    const known = this.knownFamilies(key);
    const fresh = [...new Set(families.map((family) => family.toLowerCase()))].filter(
      (family) => !known.has(family),
    );
    for (const family of fresh) known.add(family);
    if (fresh.length === 0 || !(await this.hasFaces(fresh))) {
      const current = await this.entries.get(key);
      const replaced = current?.kind !== 'ready' || current.pooled !== pooled;
      return replaced && current !== undefined ? current : null;
    }
    const entry = this.start(template);
    this.entries.delete(key);
    this.entries.set(key, entry);
    pooled.worker.retire();
    return entry;
  }

  /** Stops every Worker. */
  destroy(): void {
    for (const entry of this.entries.values()) {
      void entry.then((settled) => {
        if (settled.kind === 'ready') settled.pooled.worker.terminate();
      });
    }
    this.entries.clear();
  }

  private async start(template: TemplateRef): Promise<PoolEntry> {
    const compiled = compileTemplate(template.source);
    if (!compiled.ok) {
      return { kind: 'failed', failure: new TemplateFailure('exception', compiled.problems) };
    }
    try {
      const worker = await TemplateWorker.start(
        templateWorkerSource(compiled.compiled),
        this.options.createWorker ?? blobWorkerFactory,
        this.options.loadWatchdogMs ?? TEMPLATE_LOAD_WATCHDOG_MS,
      );
      const pooled: PooledWorker = { worker, families: new Set(), images: new Map() };
      const families = [DEFAULT_TEMPLATE_FAMILY, ...this.knownFamilies(templateKey(template))];
      await this.provideFonts(pooled, families, TEMPLATE_LOAD_WATCHDOG_MS).catch(() => false);
      return { kind: 'ready', pooled };
    } catch (error) {
      const failure =
        error instanceof TemplateFailure
          ? error
          : new TemplateFailure('crashed', [{ message: String(error), line: null, column: null }]);
      return { kind: 'failed', failure };
    }
  }

  private knownFamilies(key: string): Set<string> {
    let families = this.familiesOf.get(key);
    if (families === undefined) {
      families = new Set();
      this.familiesOf.set(key, families);
    }
    return families;
  }

  /** Whether the font catalogue has faces for any of `families`. */
  private async hasFaces(families: readonly string[]): Promise<boolean> {
    const source = this.options.fonts;
    if (source === undefined) return false;
    const faces = await Promise.all(families.map((family) => source(family).catch(() => [])));
    return faces.some((found) => found.length > 0);
  }

  private async sendImages(
    pooled: PooledWorker,
    assets: readonly AssetId[],
    watchdogMs: number,
  ): Promise<void> {
    const source = this.options.images;
    if (source === undefined) return;
    const loaded = await Promise.all(
      assets.map(async (assetId) => ({ assetId, bitmap: await source(assetId).catch(() => null) })),
    );
    const images: { id: string; bitmap: ImageBitmap }[] = [];
    for (const { assetId, bitmap } of loaded) {
      if (bitmap === null) continue;
      pooled.images.set(assetId, { width: bitmap.width, height: bitmap.height });
      images.push({ id: assetId, bitmap });
    }
    if (images.length > 0) await pooled.worker.sendImages(images, watchdogMs);
  }

  private trim(): void {
    while (this.entries.size > this.maxWorkers) {
      const [oldest] = this.entries.keys();
      if (oldest === undefined) return;
      const entry = this.entries.get(oldest);
      this.entries.delete(oldest);
      void entry?.then((settled) => {
        if (settled.kind === 'ready') settled.pooled.worker.terminate();
      });
    }
  }
}

/** The pool's key of a template version. */
function templateKey(template: TemplateRef): string {
  return `${template.id}@${String(template.version)}`;
}
