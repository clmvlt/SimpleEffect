import type { Flicks } from '@se/protocol';
import type {
  CompositorBackground,
  PreviewEngine,
  PreviewState,
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
  destroyed = false;
  private state: PreviewState = { status: 'idle', issues: [], error: null };
  private readonly listeners = new Set<(state: PreviewState) => void>();

  constructor(readonly request: PreviewEngineRequest) {}

  setScene(scene: RenderScene | null): void {
    this.scene = scene;
    this.sceneUpdates += 1;
  }
  setTime(time: Flicks): void {
    this.time = time;
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
  getState(): PreviewState {
    return this.state;
  }
  subscribe(listener: (state: PreviewState) => void): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }
  stats() {
    return {
      open: { videoFrames: 0, images: 0 },
      cachedPictures: 0,
      cachedBytes: 0,
      openSources: 0,
    };
  }
  settled(): Promise<void> {
    return Promise.resolve();
  }
  destroy(): void {
    this.destroyed = true;
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
