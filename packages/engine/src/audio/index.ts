/**
 * `@se/engine/audio`: the sound of playback — Web Audio graph (clip → clip gain → track gain/pan
 * → master), clip gain envelopes (volume curves, fades, constant-power cross-fades), one-second
 * chunks decoded by Mediabunny, scheduling 200 ms ahead of the audio clock
 * (the master clock), audio scrub and the master level meter (AudioWorklet). Loads Mediabunny:
 * import it lazily from UI code.
 */
export { createAudioEngine, type AudioEngineOptions } from './create-audio-engine';
export {
  createAudioEngineWith,
  type AudioEngine,
  type AudioEngineParts,
  type AudioEngineStats,
  type AudioScene,
} from './audio-engine';
export { audioContextClock } from './audio-clock';
export { AudioGraph, scheduleVoiceGain } from './audio-graph';
export { AudioMeter, SILENT_LEVELS, type MeterLevels } from './meter/audio-meter';
export { AudioScheduler, SCHEDULE_AHEAD_SECONDS, SCRUB_SECONDS } from './audio-scheduler';
export {
  audioClipsOf,
  type AudioClip,
  type PlayableSequence,
  type PlayableTrack,
  type TimeSpan,
} from './audio-clips';
export { GAIN_STEP_SECONDS, clipGainAt, gainEnvelope, type GainPoint } from './clip-gain';
export { dbToGain, gainToDb, stripSettings, type MixTrack, type StripSettings } from './mix';
export {
  createMediabunnyAudioSource,
  MediabunnyAudioSource,
  type AudioChunkReader,
  type AudioReaderFactory,
} from './audio-source';
