import type { AudioEngine } from '@se/engine/audio';
import type { MediaLocator } from '@se/engine/preview';

/** What the viewer gives the audio engine it starts. */
export interface AudioEngineRequest {
  readonly locator: MediaLocator;
}

/** Starts the playback audio engine (injected so tests use a fake). */
export type AudioEngineFactory = (request: AudioEngineRequest) => Promise<AudioEngine>;

/** This environment has no Web Audio: playback runs silent on the performance clock. */
export class AudioUnsupportedError extends Error {
  constructor() {
    super('Web Audio is not available: playback runs without sound.');
    this.name = 'AudioUnsupportedError';
  }
}

/**
 * The real audio engine: `@se/engine/audio` (Mediabunny, Web Audio, the meter worklet) is
 * imported only now, and environments without Web Audio (jsdom) never touch it.
 */
export const createAudioEngine: AudioEngineFactory = async (request) => {
  if (typeof AudioContext === 'undefined') throw new AudioUnsupportedError();
  const { createAudioEngine: create } = await import('@se/engine/audio');
  return create(request);
};
