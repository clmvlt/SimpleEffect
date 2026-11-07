import type { MediaLocator } from '../decode/media-variant';
import { audioContextClock } from './audio-clock';
import { createAudioEngineWith, type AudioEngine } from './audio-engine';
import { createMediabunnyAudioSource } from './audio-source';
import { AudioMeter } from './meter/audio-meter';
import meterProcessorUrl from './meter/meter-processor.ts?worker&url';

/** Options of {@link createAudioEngine}. */
export interface AudioEngineOptions {
  readonly locator: MediaLocator;
  /** An existing context (tests); default: a new interactive-latency `AudioContext`. */
  readonly context?: AudioContext;
}

/**
 * Creates the playback audio engine on a new `AudioContext`, its clock and its level meter (an
 * AudioWorklet bundled as its own file). Without the meter (worklet refused), sound still plays
 * and the meter stays silent.
 */
export async function createAudioEngine(options: AudioEngineOptions): Promise<AudioEngine> {
  const context = options.context ?? new AudioContext({ latencyHint: 'interactive' });
  const meter = await AudioMeter.create(context, meterProcessorUrl).catch(() => null);
  return createAudioEngineWith({
    context,
    clock: audioContextClock(context),
    meter,
    locator: options.locator,
    createReader: createMediabunnyAudioSource,
  });
}
