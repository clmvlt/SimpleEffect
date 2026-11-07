import type { MediaLocator, PreviewEngine } from '@se/engine/preview';

/** What the viewer gives the engine it starts. */
export interface PreviewEngineRequest {
  readonly canvas: HTMLCanvasElement;
  readonly locator: MediaLocator;
  readonly width: number;
  readonly height: number;
}

/** Starts a preview engine on a canvas (injected so tests use a fake engine). */
export type PreviewEngineFactory = (request: PreviewEngineRequest) => Promise<PreviewEngine>;

/** This machine (or test environment) has no WebCodecs: the engine cannot run. */
export class PreviewUnsupportedError extends Error {
  constructor() {
    super('WebCodecs is not available: the preview cannot decode video here.');
    this.name = 'PreviewUnsupportedError';
  }
}

/**
 * The real engine: `@se/engine/preview` (PixiJS + Mediabunny) is imported only now, so screens
 * without a viewer never load it, and environments without WebCodecs (jsdom) never touch it.
 */
export const createPreviewEngine: PreviewEngineFactory = async (request) => {
  if (typeof VideoDecoder === 'undefined') throw new PreviewUnsupportedError();
  const { createPreviewEngine: create } = await import('@se/engine/preview');
  return create(request);
};
