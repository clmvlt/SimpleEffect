import type { Flicks, FrameRate } from '@se/protocol';

import type { DecoderPool } from '../decode/decoder-pool';
import type { MediaLocator } from '../decode/media-variant';
import { PlaybackStatsCounter, type PlaybackStats } from '../playback/playback-stats';
import type { RenderScene } from './frame-renderer';
import { prefetchRequests, prefetchSpans } from './prefetch-plan';
import { PrefetchWindow } from './prefetch-window';

/** The playback a preview engine follows (`PreviewEngine.setPlayback`). */
export interface PreviewPlayback {
  /** Non-zero: 1 real time, negative in reverse. */
  readonly rate: number;
  /** The sequence's frame rate (frame counting for the HUD). */
  readonly frameRate: FrameRate;
}

/** What the prefetcher needs to know about the frame on screen. */
export interface PrefetchInput {
  readonly scene: RenderScene;
  readonly time: Flicks;
  readonly proxy: boolean;
  readonly locator: MediaLocator;
  readonly playback: PreviewPlayback;
}

/** Playback counters plus the current prefetch window, for the performance HUD. */
export interface PlaybackPrefetchStats extends PlaybackStats {
  readonly prefetchSeconds: number;
}

/**
 * Keeps the decoder pool's sequential readers ahead of playback (adaptive window, upcoming clips
 * included) and counts the frames drawn and skipped. One per preview engine.
 */
export class PlaybackPrefetcher {
  private readonly window = new PrefetchWindow();
  private readonly counter = new PlaybackStatsCounter();

  constructor(
    private readonly pool: DecoderPool,
    private readonly now: () => number,
  ) {}

  /** A new playback run starts: counters restart. */
  start(): void {
    this.counter.reset();
  }

  /** Playback jumped (seek): the next frame is not compared with the previous one. */
  jumped(): void {
    this.counter.discontinuity();
  }

  /** Points the readers at the window ahead of `input.time`. */
  update(input: PrefetchInput): void {
    const { rate } = input.playback;
    const spans = prefetchSpans(input.scene.sequence, input.time, rate, this.window.seconds);
    const requests = prefetchRequests(spans, {
      assets: input.scene.assets,
      proxy: input.proxy,
      locator: input.locator,
      direction: rate > 0 ? 1 : -1,
      maxFrames: this.pool.readAheadFrames(spans.length),
    });
    this.pool.prefetch(requests);
  }

  /** A playback frame had to wait for decoding. */
  late(): void {
    this.window.late(this.now());
  }

  /** A playback frame was drawn at sequence `time`; `waited` when it was not ready in time. */
  drawn(time: Flicks, playback: PreviewPlayback, waited: boolean): void {
    const now = this.now();
    if (!waited) this.window.onTime(now);
    this.counter.frameDrawn(time, playback.frameRate, playback.rate, now);
  }

  /** Playback stopped: readers stop (their frames stay cached). */
  stop(): void {
    this.pool.prefetch([]);
  }

  stats(): PlaybackPrefetchStats {
    return { ...this.counter.stats(this.now()), prefetchSeconds: this.window.seconds };
  }
}
