import type { Flicks } from '@se/protocol';
import type {
  CompositorBackground,
  DecoderPoolStats,
  MediaLocator,
  PreviewEngine,
  PreviewState,
  RenderScene,
} from '@se/engine/preview';

import { PreviewUnsupportedError, type PreviewEngineFactory } from './preview-engine-factory';
import type { Size } from './viewer-layout';

/** The viewer engine's lifecycle as the panel shows it. */
export type ViewerEngineStatus =
  | { readonly kind: 'starting' }
  | { readonly kind: 'running'; readonly preview: PreviewState }
  | { readonly kind: 'failed'; readonly reason: 'unsupported' | 'error'; readonly message: string };

const STARTING: ViewerEngineStatus = { kind: 'starting' };

/** What the viewer asks the engine to draw; kept while the engine starts, then applied. */
interface Settings {
  scene: RenderScene | null;
  time: Flicks;
  proxy: boolean;
  background: CompositorBackground;
  pixels: Size;
  locator: MediaLocator;
}

/**
 * Owns the viewer's engine outside React: `attach` mounts a fresh canvas in a host element and
 * starts an engine on it, `detach` destroys both (a canvas is never reused: a lost WebGL context
 * cannot be recovered). Settings arriving while the engine starts are applied once it runs.
 */
export class ViewerEngineController {
  private engine: PreviewEngine | null = null;
  private canvas: HTMLCanvasElement | null = null;
  private unsubscribe: (() => void) | null = null;
  private generation = 0;
  private status: ViewerEngineStatus = STARTING;
  private readonly listeners = new Set<() => void>();
  private readonly settings: Settings = {
    scene: null,
    time: 0 as Flicks,
    proxy: false,
    background: 'black',
    pixels: { width: 1, height: 1 },
    locator: () => null,
  };

  constructor(private readonly factory: PreviewEngineFactory) {}

  /** Mounts a canvas in `host` and starts an engine on it. */
  attach(host: HTMLElement): void {
    this.detach();
    const generation = this.generation;
    const canvas = host.ownerDocument.createElement('canvas');
    canvas.style.display = 'block';
    canvas.style.width = '100%';
    canvas.style.height = '100%';
    host.appendChild(canvas);
    this.canvas = canvas;
    this.setStatus(STARTING);
    const request = {
      canvas,
      locator: (assetId: Parameters<MediaLocator>[0], variant: Parameters<MediaLocator>[1]) =>
        this.settings.locator(assetId, variant),
      ...this.settings.pixels,
    };
    this.factory(request).then(
      (engine) => {
        if (generation !== this.generation) engine.destroy();
        else this.start(engine);
      },
      (error: unknown) => {
        if (generation === this.generation) this.setStatus(failure(error));
      },
    );
  }

  /** Destroys the engine and removes its canvas. */
  detach(): void {
    this.generation += 1;
    this.unsubscribe?.();
    this.unsubscribe = null;
    this.engine?.destroy();
    this.engine = null;
    this.canvas?.remove();
    this.canvas = null;
  }

  setScene(scene: RenderScene | null): void {
    this.settings.scene = scene;
    this.engine?.setScene(scene);
  }

  setTime(time: Flicks): void {
    this.settings.time = time;
    this.engine?.setTime(time);
  }

  setProxy(proxy: boolean): void {
    this.settings.proxy = proxy;
    this.engine?.setProxy(proxy);
  }

  setBackground(background: CompositorBackground): void {
    this.settings.background = background;
    this.engine?.setBackground(background);
  }

  /** Canvas backing size in device pixels. */
  setPixels(pixels: Size): void {
    if (
      pixels.width === this.settings.pixels.width &&
      pixels.height === this.settings.pixels.height
    ) {
      return;
    }
    this.settings.pixels = pixels;
    this.engine?.resize(pixels.width, pixels.height);
  }

  /** New media URLs (a restarted core, changed assets): redraws with them. */
  setLocator(locator: MediaLocator): void {
    this.settings.locator = locator;
    this.engine?.setScene(this.settings.scene);
  }

  getStatus = (): ViewerEngineStatus => this.status;

  subscribe = (listener: () => void): (() => void) => {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  };

  /** Decoder counters of the running engine (dev leak check); `null` before it runs. */
  stats(): DecoderPoolStats | null {
    return this.engine?.stats() ?? null;
  }

  private start(engine: PreviewEngine): void {
    this.engine = engine;
    const { pixels, proxy, background, scene, time } = this.settings;
    engine.resize(pixels.width, pixels.height);
    engine.setProxy(proxy);
    engine.setBackground(background);
    engine.setScene(scene);
    engine.setTime(time);
    this.unsubscribe = engine.subscribe((preview) => {
      this.setStatus({ kind: 'running', preview });
    });
    this.setStatus({ kind: 'running', preview: engine.getState() });
  }

  private setStatus(status: ViewerEngineStatus): void {
    this.status = status;
    for (const listener of this.listeners) listener();
  }
}

function failure(error: unknown): ViewerEngineStatus {
  return {
    kind: 'failed',
    reason: error instanceof PreviewUnsupportedError ? 'unsupported' : 'error',
    message: error instanceof Error ? error.message : String(error),
  };
}
