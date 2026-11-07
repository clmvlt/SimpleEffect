import type { AssetId } from '@se/protocol';

import type { Picture } from '../composite/compositor';
import { DecodeQueue } from './decode-queue';
import { FrameCache, type PictureLease } from './frame-cache';
import { FrameReader } from './frame-reader';
import { pictureTracker, type FrameTracker, type OpenPictureCounts } from './frame-tracker';
import { fetchImageBitmap, type ImageLoader } from './image-loader';
import type { MediaVariant } from './media-variant';
import { planRead, type ReadWindow } from './read-plan';
import {
  createMediabunnyVideoSource,
  type DecodedVideoFrame,
  type VideoFrameSource,
  type VideoSourceFactory,
} from './video-source';

/** A file of an asset to decode, with its current URL. */
export interface MediaFileRef {
  readonly assetId: AssetId;
  readonly variant: MediaVariant;
  readonly url: string;
}

/** Options of a {@link DecoderPool}; every field has a default. */
export interface DecoderPoolOptions {
  /** Estimated bytes of cached pictures (default 256 MiB). */
  readonly cacheBudgetBytes?: number;
  /** Cached pictures, all sources together (default 48: hardware decoders stall beyond). */
  readonly maxCachedPictures?: number;
  /** Video files kept open (default 8): the least recently used idle one is closed beyond. */
  readonly maxOpenSources?: number;
  readonly createVideoSource?: VideoSourceFactory;
  readonly loadImage?: ImageLoader;
  /** Leak counter (default: the page-wide {@link pictureTracker}). */
  readonly tracker?: FrameTracker;
}

/** What a pool holds, for dev tools, the performance HUD and leak tests. */
export interface DecoderPoolStats {
  readonly open: OpenPictureCounts;
  readonly cachedPictures: number;
  readonly cachedBytes: number;
  readonly openSources: number;
  /** Sequential readers decoding ahead for playback. */
  readonly readers: number;
  /** Random-access decodes waiting or running. */
  readonly queuedDecodes: number;
}

/**
 * A stretch of a file playback will show soon (`DecoderPool.prefetch`): one per clip on screen
 * or coming up, in media seconds.
 */
export interface PrefetchRequest extends ReadWindow {
  /** The reading it continues (a clip): a new key starts a new decoder. */
  readonly key: string;
  readonly file: MediaFileRef;
}

/** Share of the cache sequential readers may fill ahead of the time on screen. */
const READ_AHEAD_SHARE = 0.75;

interface VideoSlot {
  readonly id: string;
  /** Key of its pictures in the cache. */
  readonly cacheId: string;
  readonly url: string;
  readonly source: VideoFrameSource;
  readonly queue: DecodeQueue;
  lastUse: number;
}

/** A sequential reader and the source it reads. */
interface PlaybackReader {
  readonly reader: FrameReader;
  readonly slotId: string;
}

/** A sequential reader: one per playback key and file content. */
function readerKey(request: Pick<PrefetchRequest, 'key' | 'file'>): string {
  return `${request.key}|${cacheId(request.file)}`;
}

/** The decoder of a file: one per (asset, variant). */
function sourceId(file: MediaFileRef): string {
  return `${file.assetId}:${file.variant}`;
}

/**
 * Cached pictures are keyed by URL too: a changed URL is new content (regenerated proxy, relinked
 * original), so old pictures are never served for it.
 */
function cacheId(file: MediaFileRef): string {
  return `${sourceId(file)}|${file.url}`;
}

function imageSourceId(file: MediaFileRef): string {
  return `image:${cacheId(file)}`;
}

/**
 * Decodes the pictures of a sequence (`docs/architecture.md` §8): one source per (asset, variant),
 * frame-accurate access by media time, an LRU cache of decoded pictures with a memory budget,
 * and cancellation of requests a newer one made obsolete (fast scrubbing).
 *
 * The pool owns every picture it decodes: callers borrow them through leases and release them
 * once drawn. `destroy()` closes everything not leased, and leased pictures on release.
 */
export class DecoderPool {
  private readonly cache: FrameCache;
  private readonly tracker: FrameTracker;
  private readonly videos = new Map<string, VideoSlot>();
  private readonly readers = new Map<string, PlaybackReader>();
  private readonly maxCachedPictures: number;
  private readonly images = new Map<string, Promise<void>>();
  private readonly maxOpenSources: number;
  private readonly createVideoSource: VideoSourceFactory;
  private readonly loadImage: ImageLoader;
  private useCounter = 0;
  private destroyed = false;

  constructor(options: DecoderPoolOptions = {}) {
    this.tracker = options.tracker ?? pictureTracker;
    this.maxCachedPictures = options.maxCachedPictures ?? 48;
    this.cache = new FrameCache(
      {
        budgetBytes: options.cacheBudgetBytes ?? 256 * 1024 * 1024,
        maxEntries: this.maxCachedPictures,
      },
      (picture) => {
        this.tracker.close(picture.source);
      },
    );
    this.maxOpenSources = options.maxOpenSources ?? 8;
    this.createVideoSource = options.createVideoSource ?? createMediabunnyVideoSource;
    this.loadImage = options.loadImage ?? fetchImageBitmap;
  }

  /**
   * The cached frame of `file` shown at `time` (seconds from its first frame, aimed inside the
   * wanted frame: see `frameLookupTime`), without decoding.
   */
  peekVideoFrame(file: MediaFileRef, time: number): PictureLease | null {
    if (this.destroyed) return null;
    return this.cache.lease(cacheId(file), time);
  }

  /**
   * The frame of `file` shown at `time` (seconds from its first frame), decoding it if needed.
   * Resolves `null` before the first frame or when `signal` aborted first; rejects with a
   * `DecodeError` when the file cannot be decoded.
   */
  async videoFrame(
    file: MediaFileRef,
    time: number,
    signal?: AbortSignal,
  ): Promise<PictureLease | null> {
    const cached = this.peekVideoFrame(file, time);
    if (cached !== null || this.destroyed) return cached;
    const slot = this.slotFor(file);
    return slot.queue.enqueue(() => this.decode(slot, time), signal);
  }

  /**
   * Keeps sequential readers decoding the stretches playback will show (`docs/architecture.md`
   * §8): a reader per request key, kept, extended or restarted as the window moves (`planRead`);
   * readers whose key is no longer requested stop. `prefetch([])` stops them all (pause).
   */
  prefetch(requests: readonly PrefetchRequest[]): void {
    if (this.destroyed) return;
    const wanted = new Map(requests.map((request) => [readerKey(request), request]));
    for (const [key, { reader }] of this.readers) {
      if (!wanted.has(key) || !reader.running) {
        reader.stop();
        this.readers.delete(key);
      }
    }
    const maxAhead = this.readAheadFrames(wanted.size);
    for (const [key, request] of wanted) this.follow(key, request, maxAhead);
  }

  /** Frames each of `readers` sequential readers may decode ahead (its share of the cache). */
  readAheadFrames(readers: number): number {
    return Math.max(
      2,
      Math.floor((this.maxCachedPictures * READ_AHEAD_SHARE) / Math.max(1, readers)),
    );
  }

  /**
   * The frame of `file` shown at `time` during playback: from the cache, else from the reader of
   * `key` when it is about to decode it, else by random access like `videoFrame`.
   */
  async playbackFrame(
    file: MediaFileRef,
    key: string,
    time: number,
    signal?: AbortSignal,
  ): Promise<PictureLease | null> {
    const cached = this.peekVideoFrame(file, time);
    if (cached !== null || this.destroyed) return cached;
    const reader = this.readers.get(readerKey({ key, file }))?.reader;
    if (reader !== undefined && (await reader.waitFor(time, signal))) {
      const delivered = this.peekVideoFrame(file, time);
      if (delivered !== null) return delivered;
    }
    if (signal?.aborted === true) return null;
    return this.videoFrame(file, time, signal);
  }

  /** The cached image of `file`, without loading. */
  peekImage(file: MediaFileRef): PictureLease | null {
    if (this.destroyed) return null;
    return this.cache.lease(imageSourceId(file), 0);
  }

  /**
   * The image of `file`, loaded once: concurrent requests share the load, then each leases the
   * cached bitmap. Rejects with a `DecodeError` when it cannot be loaded.
   */
  async image(file: MediaFileRef): Promise<PictureLease | null> {
    const cached = this.peekImage(file);
    if (cached !== null || this.destroyed) return cached;
    const id = imageSourceId(file);
    let loading = this.images.get(id);
    if (loading === undefined) {
      loading = this.load(id, file.url).finally(() => this.images.delete(id));
      this.images.set(id, loading);
    }
    await loading;
    return this.peekImage(file);
  }

  stats(): DecoderPoolStats {
    const cache = this.cache.stats();
    return {
      open: this.tracker.counts(),
      cachedPictures: cache.entries,
      cachedBytes: cache.bytes,
      openSources: this.videos.size,
      readers: this.readers.size,
      queuedDecodes: [...this.videos.values()].reduce((sum, slot) => sum + slot.queue.size, 0),
    };
  }

  /** Closes every source and every picture not leased (leased ones close on release). */
  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;
    for (const { reader } of this.readers.values()) reader.stop();
    this.readers.clear();
    for (const slot of this.videos.values()) slot.source.dispose();
    this.videos.clear();
    this.cache.clear();
  }

  private async decode(slot: VideoSlot, time: number): Promise<PictureLease | null> {
    const again = this.cache.lease(slot.cacheId, time);
    if (again !== null) return again;
    const decoded = await slot.source.frameAt(time);
    if (decoded === null) return null;
    return this.store(slot.cacheId, decoded);
  }

  /** Adopts a decoded frame into the cache and leases it (`null` once destroyed: it is closed). */
  private store(cacheKey: string, decoded: DecodedVideoFrame): PictureLease | null {
    const frame = this.tracker.adopt(decoded.frame, 'videoFrame');
    if (this.destroyed) {
      this.tracker.close(frame);
      return null;
    }
    const picture: Picture = {
      source: frame,
      rotation: decoded.rotation,
      mirrored: decoded.mirrored,
    };
    const bytes = frame.displayWidth * frame.displayHeight * 4;
    return this.cache.insert(cacheKey, picture, decoded.start, decoded.end, bytes);
  }

  /** Keeps, extends or restarts the reader of `key` for `request`. */
  private follow(key: string, request: PrefetchRequest, maxAhead: number): void {
    const existing = this.readers.get(key)?.reader;
    const progress =
      existing === undefined ? null : { start: existing.start, position: existing.decodedUntil };
    const decision = planRead(progress, request);
    const ahead = request.direction > 0 ? maxAhead : Infinity;
    if (decision.kind === 'keep' && existing !== undefined) {
      if (request.direction > 0) existing.extend(request.from, request.to, ahead);
      return;
    }
    existing?.stop();
    const slot = this.slotFor(request.file);
    const reader = new FrameReader({
      source: slot.source,
      start: decision.kind === 'restart' ? decision.from : request.from,
      until: decision.kind === 'restart' ? decision.to : request.to,
      maxAhead: ahead,
      deliver: (decoded) => {
        this.store(slot.cacheId, decoded)?.release();
      },
    });
    this.readers.set(key, { reader, slotId: slot.id });
  }

  private async load(id: string, url: string): Promise<void> {
    const bitmap = this.tracker.adopt(await this.loadImage(url), 'image');
    if (this.destroyed) {
      this.tracker.close(bitmap);
      return;
    }
    const picture: Picture = { source: bitmap, rotation: 0, mirrored: false };
    this.cache.insert(id, picture, 0, Infinity, bitmap.width * bitmap.height * 4).release();
  }

  /** The open source of `file`, reopened when its URL changed (new content or a new core). */
  private slotFor(file: MediaFileRef): VideoSlot {
    const id = sourceId(file);
    const existing = this.videos.get(id);
    this.useCounter += 1;
    if (existing?.url === file.url) {
      existing.lastUse = this.useCounter;
      return existing;
    }
    if (existing !== undefined) {
      existing.source.dispose();
      this.videos.delete(id);
      this.cache.dropSource(existing.cacheId);
    }
    const slot: VideoSlot = {
      id,
      cacheId: cacheId(file),
      url: file.url,
      source: this.createVideoSource(file.url),
      queue: new DecodeQueue(),
      lastUse: this.useCounter,
    };
    this.videos.set(id, slot);
    this.closeIdleSources(slot);
    return slot;
  }

  /**
   * Closes the least recently used idle sources beyond the limit, never `wanted` (their cached
   * frames stay: they are still valid pictures).
   */
  private closeIdleSources(wanted: VideoSlot): void {
    const reading = new Set([...this.readers.values()].map((entry) => entry.slotId));
    const idle = [...this.videos.values()]
      .filter((slot) => slot !== wanted && slot.queue.size === 0 && !reading.has(slot.id))
      .sort((left, right) => left.lastUse - right.lastUse);
    for (const slot of idle) {
      if (this.videos.size <= this.maxOpenSources) return;
      slot.source.dispose();
      this.videos.delete(slot.id);
    }
  }
}
