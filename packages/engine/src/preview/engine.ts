import type { Flicks } from '@se/protocol';

import type { Compositor, CompositorBackground, RenderCanvas } from '../composite/compositor';
import type { DecoderPool, DecoderPoolOptions, DecoderPoolStats } from '../decode/decoder-pool';
import type { MediaLocator } from '../decode/media-variant';
import { FrameRenderer, type RenderScene } from './frame-renderer';
import type { MediaIssue } from './media-issue';

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

/**
 * The viewer's engine, living outside React (a thin hook creates and destroys it): it keeps the
 * scene, time and options, coalesces changes into one redraw per animation frame and reports
 * its state.
 */
export interface PreviewEngine {
  setScene(scene: RenderScene | null): void;
  setTime(time: Flicks): void;
  setProxy(enabled: boolean): void;
  setBackground(background: CompositorBackground): void;
  /** Canvas size in pixels (the frame is scaled to fill it). */
  resize(width: number, height: number): void;
  getState(): PreviewState;
  /** Calls `listener` on every state change; returns the unsubscribe function. */
  subscribe(listener: (state: PreviewState) => void): () => void;
  /** Decoder pool counters (open frames: the dev leak check). */
  stats(): DecoderPoolStats;
  /** Resolves once the latest requested frame is drawn (or the engine is destroyed). */
  settled(): Promise<void>;
  /** Frees decoders, frames and GPU resources. */
  destroy(): void;
}

function defaultSchedule(callback: () => void): void {
  if (typeof requestAnimationFrame === 'function') requestAnimationFrame(callback);
  else setTimeout(callback, 0);
}

const EMPTY_SCENE: RenderScene = {
  sequence: { format: { width: 16, height: 9 }, tracks: [] },
  assets: {},
};

class Engine implements PreviewEngine {
  private scene: RenderScene | null = null;
  private time = 0 as Flicks;
  private proxy = false;
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
    locator: MediaLocator,
    private readonly schedule: (callback: () => void) => void,
  ) {
    this.renderer = new FrameRenderer(compositor, pool, locator);
  }

  setScene(scene: RenderScene | null): void {
    this.scene = scene;
    this.invalidate();
  }

  setTime(time: Flicks): void {
    if (time === this.time) return;
    this.time = time;
    this.invalidate();
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

  stats() {
    return this.pool.stats();
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

  private async draw(): Promise<void> {
    if (this.destroyed) return;
    this.inFlight += 1;
    try {
      const request = { scene: this.scene ?? EMPTY_SCENE, time: this.time, proxy: this.proxy };
      const outcome = await this.renderer.render(request, () => {
        this.setState({ ...this.state, status: 'loading' });
      });
      // After `destroy`, listeners are gone: a late state change reaches no one.
      if (outcome.kind === 'drawn') {
        this.setState({ status: 'ready', issues: outcome.issues, error: null });
      }
    } catch (error: unknown) {
      const message = error instanceof Error ? error.message : String(error);
      this.setState({ status: 'error', issues: [], error: message });
    } finally {
      this.inFlight -= 1;
      if (this.isSettled()) this.wakeWaiters();
    }
  }

  private setState(state: PreviewState): void {
    this.state = state;
    for (const listener of this.listeners) listener(state);
  }

  private wakeWaiters(): void {
    const waiters = this.waiters;
    this.waiters = [];
    for (const wake of waiters) wake();
  }
}

/** Assembles an engine from its parts (tests inject a fake compositor or pool). */
export function createEngineWith(
  compositor: Compositor,
  pool: DecoderPool,
  locator: MediaLocator,
  schedule: (callback: () => void) => void = defaultSchedule,
): PreviewEngine {
  return new Engine(compositor, pool, locator, schedule);
}
