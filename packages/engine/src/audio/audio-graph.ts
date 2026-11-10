import type { TrackId } from '@se/protocol';

import type { AudioOutput, AudioVoice, AudioVoiceRequest } from './audio-scheduler';
import { stripSettings, type MixTrack } from './mix';

/** Time constant of gain and pan changes: immediate to the ear, without clicks. */
const SMOOTHING_SECONDS = 0.01;

/** The nodes of one track: gain (volume, mute, solo) then stereo balance. */
interface Strip {
  readonly gain: GainNode;
  readonly pan: StereoPannerNode;
}

/**
 * The Web Audio graph (`docs/architecture.md` §8): clip voice → clip gain → track gain → track
 * pan → master gain → speakers, with taps on the master (level meter). Track settings apply with
 * a 10 ms smoothing, so mute, solo and volume act immediately during playback.
 */
export class AudioGraph implements AudioOutput {
  readonly master: GainNode;
  private readonly strips = new Map<TrackId, Strip>();

  /** `destination` defaults to the speakers (an offline render or a test tap otherwise). */
  constructor(
    private readonly context: BaseAudioContext,
    destination: AudioNode = context.destination,
  ) {
    this.master = context.createGain();
    this.master.connect(destination);
  }

  currentTime(): number {
    return this.context.currentTime;
  }

  /**
   * Applies the mixing state of the sequence's tracks; strips of removed tracks are freed.
   * `immediate` sets the levels at once instead of smoothing them: an offline render starts from
   * them (export blocks, which would otherwise fade every muted track in and out at each block).
   */
  setMix(tracks: readonly MixTrack[], { immediate = false }: { immediate?: boolean } = {}): void {
    const settings = stripSettings(tracks);
    const now = this.context.currentTime;
    for (const [trackId, strip] of this.strips) {
      if (settings.has(trackId)) continue;
      strip.gain.disconnect();
      strip.pan.disconnect();
      this.strips.delete(trackId);
    }
    for (const [trackId, { gain, pan }] of settings) {
      const strip = this.stripOf(trackId);
      if (immediate) {
        strip.gain.gain.value = gain;
        strip.pan.pan.value = pan;
        continue;
      }
      strip.gain.gain.setTargetAtTime(gain, now, SMOOTHING_SECONDS);
      strip.pan.pan.setTargetAtTime(pan, now, SMOOTHING_SECONDS);
    }
  }

  /** Sends the master to `node` too (a meter); the node's output, if any, is not heard twice. */
  tap(node: AudioNode): void {
    this.master.connect(node);
  }

  play(request: AudioVoiceRequest, onEnded: () => void): AudioVoice {
    const source = this.context.createBufferSource();
    source.buffer = request.buffer;
    source.playbackRate.value = request.playbackRate;
    const gain = this.context.createGain();
    scheduleVoiceGain(gain.gain, request);
    source.connect(gain);
    gain.connect(this.stripOf(request.trackId).gain);
    source.onended = () => {
      source.disconnect();
      gain.disconnect();
      onEnded();
    };
    source.start(request.when, request.offset, request.duration);
    return {
      stop: () => {
        source.stop();
      },
    };
  }

  /** Disconnects everything (the context is closed by its owner). */
  dispose(): void {
    for (const strip of this.strips.values()) {
      strip.gain.disconnect();
      strip.pan.disconnect();
    }
    this.strips.clear();
    this.master.disconnect();
  }

  private stripOf(trackId: TrackId): Strip {
    const existing = this.strips.get(trackId);
    if (existing !== undefined) return existing;
    const gain = this.context.createGain();
    const pan = this.context.createStereoPanner();
    gain.connect(pan);
    pan.connect(this.master);
    const strip = { gain, pan };
    this.strips.set(trackId, strip);
    return strip;
  }
}

/**
 * A voice's gain: its envelope (constant, or linear ramps between its points), or for scrub
 * snippets its level faded in and out so they never click.
 */
export function scheduleVoiceGain(
  param: Pick<AudioParam, 'value' | 'setValueAtTime' | 'linearRampToValueAtTime'>,
  request: AudioVoiceRequest,
): void {
  const [first, ...rest] = request.gain;
  const level = first?.gain ?? 1;
  if (request.fade > 0) {
    const end = request.when + request.duration / request.playbackRate;
    const fade = Math.min(request.fade, (end - request.when) / 2);
    param.setValueAtTime(0, request.when);
    param.linearRampToValueAtTime(level, request.when + fade);
    param.setValueAtTime(level, end - fade);
    param.linearRampToValueAtTime(0, end);
    return;
  }
  if (rest.length === 0) {
    param.value = level;
    return;
  }
  param.setValueAtTime(level, request.when + (first?.at ?? 0));
  for (const point of rest) param.linearRampToValueAtTime(point.gain, request.when + point.at);
}
