/** @vitest-environment jsdom */
import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { fakeEngineFactory } from '../testing/fake-preview-engine';
import { PreviewUnsupportedError } from './preview-engine-factory';
import { ViewerEngineController } from './viewer-engine-controller';

const scene = { sequence: { format: { width: 1920, height: 1080 }, tracks: [] }, assets: {} };

function flush(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

describe('ViewerEngineController', () => {
  it('mounts a canvas, starts the engine and applies the settings given meanwhile', async () => {
    const { factory, engines } = fakeEngineFactory();
    const controller = new ViewerEngineController(factory);
    const host = document.createElement('div');
    controller.setPixels({ width: 960, height: 540 });
    controller.attach(host);
    controller.setScene(scene);
    controller.setTime(42 as Flicks);
    controller.setProxy(true);
    controller.setBackground('transparent');
    expect(controller.getStatus()).toEqual({ kind: 'starting' });
    await flush();
    const [engine] = engines;
    expect(engine?.request.canvas.parentElement).toBe(host);
    expect(engine?.request).toMatchObject({ width: 960, height: 540 });
    expect(engine).toMatchObject({ scene, time: 42, proxy: true, background: 'transparent' });
    expect(controller.getStatus()).toMatchObject({ kind: 'running' });
  });

  it('forwards changes to the running engine and redraws on new media URLs', async () => {
    const { factory, engines } = fakeEngineFactory();
    const controller = new ViewerEngineController(factory);
    controller.attach(document.createElement('div'));
    await flush();
    const engine = engines[0];
    controller.setPixels({ width: 320, height: 180 });
    controller.setPixels({ width: 320, height: 180 });
    expect(engine?.size).toEqual([320, 180]);
    const updates = engine?.sceneUpdates ?? 0;
    controller.setLocator(() => 'http://127.0.0.1/media');
    expect(engine?.sceneUpdates).toBe(updates + 1);
    expect(engine?.request.locator('a', 'original')).toBe('http://127.0.0.1/media');
    engine?.emit({ status: 'loading', issues: [], error: null });
    expect(controller.getStatus()).toEqual({
      kind: 'running',
      preview: { status: 'loading', issues: [], error: null },
    });
  });

  it('destroys the engine and its canvas on detach, even while it starts', async () => {
    const { factory, engines } = fakeEngineFactory();
    const controller = new ViewerEngineController(factory);
    const host = document.createElement('div');
    controller.attach(host);
    controller.detach();
    controller.attach(host);
    await flush();
    expect(engines.map((engine) => engine.destroyed)).toEqual([true, false]);
    expect(host.querySelectorAll('canvas')).toHaveLength(1);
    controller.detach();
    expect(engines[1]?.destroyed).toBe(true);
    expect(host.querySelector('canvas')).toBeNull();
  });

  it('reports why the engine could not start', async () => {
    const unsupported = new ViewerEngineController(() =>
      Promise.reject(new PreviewUnsupportedError()),
    );
    unsupported.attach(document.createElement('div'));
    const broken = new ViewerEngineController(() => Promise.reject(new Error('no WebGL 2')));
    broken.attach(document.createElement('div'));
    await flush();
    expect(unsupported.getStatus()).toMatchObject({ kind: 'failed', reason: 'unsupported' });
    expect(broken.getStatus()).toEqual({ kind: 'failed', reason: 'error', message: 'no WebGL 2' });
  });
});
