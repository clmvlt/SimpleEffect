import type { Picture } from '../composite/compositor';

/**
 * A borrowed picture: valid until `release()`, which the borrower must call exactly once (extra
 * calls are ignored). The cache never closes a picture while it is leased.
 */
export interface PictureLease {
  readonly picture: Picture;
  release(): void;
}

/** Limits of a {@link FrameCache}. */
export interface FrameCacheLimits {
  /** Estimated bytes of cached pixels before the least recently used pictures are closed. */
  readonly budgetBytes: number;
  /** Cached pictures before eviction (hardware decoders stall when too many frames are held). */
  readonly maxEntries: number;
}

interface Entry {
  readonly sourceId: string;
  /** Media time range shown by the picture, in seconds: `[start, end)`. */
  readonly start: number;
  readonly end: number;
  readonly picture: Picture;
  readonly bytes: number;
  leases: number;
  /** Removed from the cache: closed once the last lease is released. */
  removed: boolean;
}

/**
 * LRU cache of decoded pictures with leases (`docs/architecture.md` §8). It owns every picture
 * inserted: evicted, dropped or cleared pictures are closed through `close` as soon as no lease
 * holds them. Lookups find the picture whose time range contains a media time.
 */
export class FrameCache {
  /** Map order is recency order: oldest first. */
  private readonly entries = new Map<string, Entry>();
  private bytes = 0;

  constructor(
    private readonly limits: FrameCacheLimits,
    private readonly close: (picture: Picture) => void,
  ) {}

  /** Leases the cached picture of `sourceId` shown at `time` (seconds), if any. */
  lease(sourceId: string, time: number): PictureLease | null {
    for (const [key, entry] of this.entries) {
      if (entry.sourceId === sourceId && entry.start <= time && time < entry.end) {
        this.entries.delete(key);
        this.entries.set(key, entry);
        return this.leaseOf(entry);
      }
    }
    return null;
  }

  /**
   * Caches a picture showing `[start, end)` of `sourceId` and leases it. A picture of the same
   * source and start already cached wins: the new one is closed at once.
   */
  insert(
    sourceId: string,
    picture: Picture,
    start: number,
    end: number,
    bytes: number,
  ): PictureLease {
    const key = `${sourceId}@${String(start)}`;
    const existing = this.entries.get(key);
    if (existing !== undefined) {
      this.close(picture);
      return this.leaseOf(existing);
    }
    const entry: Entry = { sourceId, start, end, picture, bytes, leases: 0, removed: false };
    this.entries.set(key, entry);
    this.bytes += bytes;
    const lease = this.leaseOf(entry);
    this.evict();
    return lease;
  }

  /**
   * Removes the pictures of `sourceId` shown entirely before `time` (forward playback has passed
   * them): a decoder running through a file gets its surfaces back instead of stalling.
   */
  dropBefore(sourceId: string, time: number): void {
    for (const [key, entry] of this.entries) {
      if (entry.sourceId === sourceId && entry.end <= time) this.remove(key, entry);
    }
  }

  /** Removes every picture of `sourceId` (its file changed). */
  dropSource(sourceId: string): void {
    for (const [key, entry] of this.entries) {
      if (entry.sourceId === sourceId) this.remove(key, entry);
    }
  }

  /** Removes every picture; leased ones close when released. */
  clear(): void {
    for (const [key, entry] of this.entries) this.remove(key, entry);
  }

  /** Cached pictures and their estimated bytes. */
  stats(): { readonly entries: number; readonly bytes: number } {
    return { entries: this.entries.size, bytes: this.bytes };
  }

  private leaseOf(entry: Entry): PictureLease {
    entry.leases += 1;
    let released = false;
    return {
      picture: entry.picture,
      release: () => {
        if (released) return;
        released = true;
        entry.leases -= 1;
        if (entry.removed && entry.leases === 0) this.close(entry.picture);
        else this.evict();
      },
    };
  }

  /** Closes least recently used, unleased pictures until the limits hold. */
  private evict(): void {
    for (const [key, entry] of this.entries) {
      if (this.bytes <= this.limits.budgetBytes && this.entries.size <= this.limits.maxEntries) {
        return;
      }
      if (entry.leases === 0) this.remove(key, entry);
    }
  }

  private remove(key: string, entry: Entry): void {
    this.entries.delete(key);
    this.bytes -= entry.bytes;
    entry.removed = true;
    if (entry.leases === 0) this.close(entry.picture);
  }
}
