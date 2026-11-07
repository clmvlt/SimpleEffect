import type { Flicks } from '@se/protocol';

import type { Compositor, CompositorBackground, RenderCanvas } from '../composite/compositor';
import type { DecoderPool, DecoderPoolOptions, DecoderPoolStats } from '../decode/decoder-pool';
import type { MediaLocator } from '../decode/media-variant';
import { FrameRenderer, type RenderScene } from './frame-renderer';
import type { MediaIssue } from './media-issue';
import {
  PlaybackPrefetcher,
  type PlaybackPrefetchStats,
  type PreviewPlayback,
} from './playback-prefetcher';

/** Where the preview stands, for the viewer's indicators. */
export interface PreviewState {
  /**
   * `idle`: nothing drawn yet; `loading`: waiting for pictures (the previous image stays);
   * `ready`: the requested frame is on screen; `error`: drawing failed (see `error`).
   */
  readonly status: 'idle' | 'loading' | 'ready' | 'error';
  /** Media problems of the frame on screen. */
  readonly issues: readonly MediaIssue[];
  readonly error: string | null;
}

/** Options of {@link createPreviewEngine}. */
export interface PreviewEngineOptions {
  readonly canvas: RenderCanvas;
  readonly locator: MediaLocator;
  /** Initial canvas size in pixels. */
  readonly width: number;
  readonly height: number;
  readonly renderer?: 'webgl' | 'webgpu';
  readonly background?: CompositorBackground;
  readonly pool?: DecoderPoolOptions;
  /** Read-back after compositing (tests); off by default. */
  readonly preserveDrawingBuffer?: boolean;
  /** Runs a coalesced redraw (default: the next animation frame, or a macrotask in Workers). */
  readonly schedule?: (callback: () => void) => void;
}

/** Decoder and playback counters, for the performance HUD and leak checks. */
export interface PreviewStats extends DecoderPoolStats, PlaybackPrefetchStats {}

/**
 * The viewer's engine, living outside React (a thin hook creates and destroys it): it keeps the
 * scene, time and options, coalesces changes into one redraw per animation frame and reports
 * its state.
 */
export interface PreviewEngine {
  setScene(scene: RenderScene | null): void;
  /**
   * The time to show. While playing it is drawn at once, in the caller's refresh (when its
   * pictures are ready; otherwise as soon as they are, unless a newer time came meanwhile).
   */
  setTime(time: Flicks): void;
  /**
   * Playback state (`null` when paused): decoding switches to sequential readers kept ahead of
   * the playhead. Calling it again while playing marks a jump (seek, rate change).
   */
  setPlayback(playback: PreviewPlayback | null): void;
  setProxy(enabled: boolean): void;
  setBackground(background: CompositorBackground): void;
  /** Canvas size in pixels (the frame is scaled to fill it). */
  resize(width: number, height: number): void;
  getState(): PreviewState;
  /** Calls `listener` on every state change; returns the unsubscribe function. */
  subscribe(listener: (state: PreviewState) => void): () => void;
  /** Decoder and playback counters (open frames: the dev leak check). */
  stats(): PreviewStats;
  /** Resolves once the latest requested frame is drawn (or the engine is destroyed). */
  settled(): Promise<void>;
  /** Frees decoders, frames and GPU resources. */
  destroy(): void;
}

function defaultSchedule(callback: () => void): void {
  if (typeof requestAnimationFrame === 'function') requestAnimationFrame(callback);
  else setTimeout(callback, 0);
}

function sameIssue(left: MediaIssue, right: MediaIssue): boolean {
  return (
    left.assetId === right.assetId &&
    left.kind === right.kind &&
    left.failure === right.failure &&
    left.message === right.message
  );
}

function sameState(left: PreviewState, right: PreviewState): boolean {
  return (
    left.status === right.status &&
    left.error === right.error &&
    left.issues.length === right.issues.length &&
    left.issues.every((issue, index) => {
      const other = right.issues[index];
      return other !== undefined && sameIssue(issue, other);
    })
  );
}

const EMPTY_SCENE: RenderScene = {
  sequence: { format: { width: 16, height: 9 }, tracks: [] },
  assets: {},
};

class Engine implements PreviewEngine {
  private scene: RenderScene | null = null;
  private time = 0 as Flicks;
  private proxy = false;
  private playback: PreviewPlayback | null = null;
  private drawnTime: Flicks | null = null;
  private readonly prefetcher: PlaybackPrefetcher;
  private state: PreviewState = { status: 'idle', issues: [], error: null };
  private readonly listeners = new Set<(state: PreviewState) => void>();
  private readonly renderer: FrameRenderer;
  private scheduled = false;
  private inFlight = 0;
  private waiters: (() => void)[] = [];
  private destroyed = false;

  constructor(
    private readonly compositor: Compositor,
    private readonly pool: DecoderPool,
    private readonly locator: MediaLocator,
    private readonly schedule: (callback: () => void) => void,
    now: () => number,
  ) {
    this.renderer = new FrameRenderer(compositor, pool, locator);
    this.prefetcher = new PlaybackPrefetcher(pool, now);
  }

  setScene(scene: RenderScene | null): void {
    this.scene = scene;
    this.invalidate();
  }

  setTime(time: Flicks): void {
    if (time === this.time) return;
    this.time = time;
    if (this.playback === null) {
      this.invalidate();
      return;
    }
    this.prefetch();
    this.drawPlaybackFrame();
  }

  setPlayback(playback: PreviewPlayback | null): void {
    if (this.destroyed) return;
    const wasPlaying = this.playback !== null;
    this.playback = playback;
    if (playback === null) {
      if (wasPlaying) this.prefetcher.stop();
      return;
    }
    if (wasPlaying) this.prefetcher.jumped();
    else this.prefetcher.start();
    this.prefetch();
  }

  setProxy(enabled: boolean): void {
    if (enabled === this.proxy) return;
    this.proxy = enabled;
    this.invalidate();
  }

  setBackground(background: CompositorBackground): void {
    this.compositor.setBackground(background);
    this.invalidate();
  }

  resize(width: number, height: number): void {
    this.compositor.resize(width, height);
    this.invalidate();
  }

  getState(): PreviewState {
    return this.state;
  }

  subscribe(listener: (state: PreviewState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  stats(): PreviewStats {
    return { ...this.pool.stats(), ...this.prefetcher.stats() };
  }

  settled(): Promise<void> {
    if (this.isSettled()) return Promise.resolve();
    return new Promise((resolve) => this.waiters.push(resolve));
  }

  destroy(): void {
    if (this.destroyed) return;
    this.destroyed = true;
    this.renderer.cancel();
    this.pool.destroy();
    this.compositor.destroy();
    this.listeners.clear();
    this.wakeWaiters();
  }

  private isSettled(): boolean {
    return this.destroyed || (!this.scheduled && this.inFlight === 0);
  }

  private invalidate(): void {
    if (this.scheduled || this.destroyed) return;
    this.scheduled = true;
    this.schedule(() => {
      this.scheduled = false;
      void this.draw();
    });
  }

  private prefetch(): void {
    const { playback, scene } = this;
    if (playback === null || scene === null) return;
    this.prefetcher.update({
      scene,
      time: this.time,
      proxy: this.proxy,
      locator: this.locator,
      playback,
    });
  }

  /**
   * Playback draws one frame at a time: a frame still waiting for its pictures is not superseded
   * (that could starve the screen when decoding lags); the latest time is drawn once it is done.
   */
  private drawPlaybackFrame(): void {
    if (this.inFlight > 0 || this.destroyed) return;
    void this.draw();
  }

  private async draw(): Promise<void> {
    if (this.destroyed) return;
    this.inFlight += 1;
    const playback = this.playback;
    const time = this.time;
    let waited = false;
    try {
      const request = {
        scene: this.scene ?? EMPTY_SCENE,
        time,
        proxy: this.proxy,
        playback: playback !== null,
      };
      const outcome = await this.renderer.render(request, () => {
        waited = true;
        // During playback a late frame is skipped over, not announced: no spinner flicker.
        if (playback === null) this.setState({ ...this.state, status: 'loading' });
        else this.prefetcher.late();
      });
      // After `destroy`, listeners are gone: a late state change reaches no one.
      if (outcome.kind === 'drawn') {
        this.drawnTime = time;
        if (playback !== null) this.prefetcher.drawn(time, playback, waited);
        this.setState({ status: 'ready', issues: outcome.issues, error: null });
      }
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      this.setState({ status: 'error', issues: [], error: message });
    } finally {
      this.inFlight -= 1;
      if (this.playback !== null && this.drawnTime !== this.time) this.drawPlaybackFrame();
      if (this.isSettled()) this.wakeWaiters();
    }
  }

  private setState(state: PreviewState): void {
    // Playback draws many frames per second: only real changes reach the viewer (no re-renders).
    if (sameState(state, this.state)) return;
    this.state = state;
    for (const listener of this.listeners) listener(state);
  }

  private wakeWaiters(): void {
    const waiters = this.waiters;
    this.waiters = [];
    for (const wake of waiters) wake();
  }
}

/** Assembles an engine from its parts (tests inject a fake compositor or pool, and clocks). */
export function createEngineWith(
  compositor: Compositor,
  pool: DecoderPool,
  locator: MediaLocator,
  schedule: (callback: () => void) => void = defaultSchedule,
  now: () => number = () => performance.now(),
): PreviewEngine {
  return new Engine(compositor, pool, locator, schedule, now);
}
