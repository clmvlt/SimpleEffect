import type { Flicks } from '@se/protocol';
import type {
  CompositorBackground,
  FontLoader,
  FrameLayout,
  PreviewEngine,
  PreviewPlayback,
  PreviewState,
  PreviewStats,
  RenderScene,
} from '@se/engine/preview';

import type {
  PreviewEngineFactory,
  PreviewEngineRequest,
} from '../services/preview-engine-factory';

/** A preview engine recording what the viewer asks of it (no GPU, no decoding). */
export class FakePreviewEngine implements PreviewEngine {
  scene: RenderScene | null = null;
  time: Flicks | null = null;
  proxy = false;
  background: CompositorBackground = 'black';
  size: readonly [number, number] = [0, 0];
  sceneUpdates = 0;
  playback: PreviewPlayback | null = null;
  /** Every time set, in order (playback draws). */
  readonly times: Flicks[] = [];
  destroyed = false;
  fonts: FontLoader | null = null;
  private state: PreviewState = { status: 'idle', issues: [], missingFonts: [], error: null };
  private readonly listeners = new Set<(state: PreviewState) => void>();
  private layout: FrameLayout | null = null;
  private readonly layoutListeners = new Set<(layout: FrameLayout) => void>();

  constructor(readonly request: PreviewEngineRequest) {}

  setScene(scene: RenderScene | null): void {
    this.scene = scene;
    this.sceneUpdates += 1;
  }
  setTime(time: Flicks): void {
    this.time = time;
    this.times.push(time);
  }
  setPlayback(playback: PreviewPlayback | null): void {
    this.playback = playback;
  }
  setProxy(enabled: boolean): void {
    this.proxy = enabled;
  }
  setBackground(background: CompositorBackground): void {
    this.background = background;
  }
  resize(width: number, height: number): void {
    this.size = [width, height];
  }
  setFonts(fonts: FontLoader | null): void {
    this.fonts = fonts;
  }
  getLayout(): FrameLayout | null {
    return this.layout;
  }
  subscribeLayout(listener: (layout: FrameLayout) => void): () => void {
    this.layoutListeners.add(listener);
    return () => this.layoutListeners.delete(listener);
  }
  getState(): PreviewState {
    return this.state;
  }
  subscribe(listener: (state: PreviewState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
  stats(): PreviewStats {
    return {
      open: { videoFrames: 0, images: 0 },
      cachedPictures: 3,
      cachedBytes: 24 * 1024 * 1024,
      openSources: 1,
      readers: 1,
      queuedDecodes: 0,
      framesPerSecond: this.playback === null ? 0 : 25,
      droppedFrames: 2,
      prefetchSeconds: 0.5,
    };
  }
  settled(): Promise<void> {
    return Promise.resolve();
  }
  destroy(): void {
    this.destroyed = true;
  }

  /** Simulates a drawn frame with this layout (the gizmo's input). */
  emitLayout(layout: FrameLayout): void {
    this.layout = layout;
    for (const listener of this.layoutListeners) listener(layout);
  }

  /** Simulates an engine state change (loading, issues…). */
  emit(state: PreviewState): void {
    this.state = state;
    for (const listener of this.listeners) listener(state);
  }
}

/** A factory handing out {@link FakePreviewEngine}s, keeping every engine it made. */
export function fakeEngineFactory(): {
  readonly factory: PreviewEngineFactory;
  readonly engines: FakePreviewEngine[];
} {
  const engines: FakePreviewEngine[] = [];
  return {
    engines,
    factory: (request) => {
      const engine = new FakePreviewEngine(request);
      engines.push(engine);
      return Promise.resolve(engine);
    },
  };
}
