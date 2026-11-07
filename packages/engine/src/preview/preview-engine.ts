import { createPixiCompositor } from '../composite/pixi/pixi-compositor';
import { DecoderPool } from '../decode/decoder-pool';
import { createEngineWith, type PreviewEngine, type PreviewEngineOptions } from './engine';

/** Creates the engine drawing on `options.canvas`; rejects when no GPU context is available. */
export async function createPreviewEngine(options: PreviewEngineOptions): Promise<PreviewEngine> {
  const compositor = await createPixiCompositor({
    canvas: options.canvas,
    width: options.width,
    height: options.height,
    ...(options.renderer === undefined ? {} : { renderer: options.renderer }),
    ...(options.background === undefined ? {} : { background: options.background }),
    ...(options.preserveDrawingBuffer === undefined
      ? {}
      : { preserveDrawingBuffer: options.preserveDrawingBuffer }),
  });
  const engine = createEngineWith(
    compositor,
    new DecoderPool(options.pool),
    options.locator,
    options.schedule,
  );
  if (options.fonts !== undefined) engine.setFonts(options.fonts);
  return engine;
}
