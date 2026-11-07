import type { AssetId } from '@se/protocol';

import type { AudioChunk } from './audio-chunk';
import type { AudioChunkProvider } from './audio-scheduler';
import type { AudioChunkReader, AudioReaderFactory } from './audio-source';

/** The URL of the file whose sound an asset plays, or `null` (no sound to play now). */
export type AudioFileLocator = (assetId: AssetId) => string | null;

/** Options of an {@link AudioChunkPool}. */
export interface AudioChunkPoolOptions {
  readonly createReader: AudioReaderFactory;
  /** Decoded sound kept in memory (default 64 MiB, about 3 minutes of 48 kHz stereo). */
  readonly budgetBytes?: number;
  /** Files kept open (default 8). */
  readonly maxOpenReaders?: number;
}

/** What an audio chunk pool holds, for the performance HUD. */
export interface AudioChunkPoolStats {
  readonly chunks: number;
  readonly bytes: number;
  readonly loading: number;
}

interface CachedChunk {
  readonly chunk: AudioChunk;
  readonly bytes: number;
}

/**
 * Decodes and caches the sound of assets in one-second chunks (LRU within a byte budget), one
 * reader per file. Concurrent loads of a chunk share one decode. A file that fails (no sound, a
 * codec this machine cannot decode, unreadable) is not retried until its URL changes.
 */
export class AudioChunkPool implements AudioChunkProvider {
  private locate: AudioFileLocator = () => null;
  /** Map order is recency order: oldest first. */
  private readonly cached = new Map<string, CachedChunk>();
  private readonly loading = new Map<string, Promise<AudioChunk | null>>();
  private readonly readers = new Map<string, AudioChunkReader>();
  private readonly failed = new Set<string>();
  /** Decodes running per file URL: a busy reader is never closed. */
  private readonly busy = new Map<string, number>();
  private bytes = 0;
  private destroyed = false;
  private readonly budgetBytes: number;
  private readonly maxOpenReaders: number;

  constructor(private readonly options: AudioChunkPoolOptions) {
    this.budgetBytes = options.budgetBytes ?? 64 * 1024 * 1024;
    this.maxOpenReaders = options.maxOpenReaders ?? 8;
  }

  /** Where assets' files are read (changes with the scene and the core connection). */
  setLocator(locate: AudioFileLocator): void {
    this.locate = locate;
  }

  peek(assetId: AssetId, index: number): AudioChunk | null {
    const url = this.locate(assetId);
    if (url === null) return null;
    const key = chunkKey(url, index);
    const entry = this.cached.get(key);
    if (entry === undefined) return null;
    this.cached.delete(key);
    this.cached.set(key, entry);
    return entry.chunk;
  }

  load(assetId: AssetId, index: number): Promise<AudioChunk | null> {
    const url = this.locate(assetId);
    if (url === null || this.failed.has(url) || this.destroyed || index < 0) {
      return Promise.resolve(null);
    }
    const cached = this.peek(assetId, index);
    if (cached !== null) return Promise.resolve(cached);
    const key = chunkKey(url, index);
    let pending = this.loading.get(key);
    if (pending === undefined) {
      pending = this.decode(url, index, key).finally(() => this.loading.delete(key));
      this.loading.set(key, pending);
    }
    return pending;
  }

  stats(): AudioChunkPoolStats {
    return { chunks: this.cached.size, bytes: this.bytes, loading: this.loading.size };
  }

  destroy(): void {
    this.destroyed = true;
    for (const reader of this.readers.values()) reader.dispose();
    this.readers.clear();
    this.cached.clear();
    this.bytes = 0;
  }

  private async decode(url: string, index: number, key: string): Promise<AudioChunk | null> {
    this.busy.set(url, (this.busy.get(url) ?? 0) + 1);
    try {
      const chunk = await this.readerFor(url).chunk(index);
      if (chunk !== null && !this.destroyed) this.store(key, chunk);
      return chunk;
    } catch {
      // The viewer reports unreadable media through the picture; sound just stays silent.
      if (!this.destroyed) this.failed.add(url);
      return null;
    } finally {
      const count = (this.busy.get(url) ?? 1) - 1;
      if (count > 0) this.busy.set(url, count);
      else this.busy.delete(url);
    }
  }

  private store(key: string, chunk: AudioChunk): void {
    const bytes = chunk.buffer.length * chunk.buffer.numberOfChannels * 4;
    this.cached.set(key, { chunk, bytes });
    this.bytes += bytes;
    for (const [oldKey, entry] of this.cached) {
      if (this.bytes <= this.budgetBytes) break;
      this.cached.delete(oldKey);
      this.bytes -= entry.bytes;
    }
  }

  private readerFor(url: string): AudioChunkReader {
    const existing = this.readers.get(url);
    if (existing !== undefined) {
      this.readers.delete(url);
      this.readers.set(url, existing);
      return existing;
    }
    const reader = this.options.createReader(url);
    this.readers.set(url, reader);
    for (const [oldUrl, old] of this.readers) {
      if (this.readers.size <= this.maxOpenReaders) break;
      if (oldUrl === url || this.busy.has(oldUrl)) continue;
      old.dispose();
      this.readers.delete(oldUrl);
    }
    return reader;
  }
}

function chunkKey(url: string, index: number): string {
  return `${url}#${String(index)}`;
}
