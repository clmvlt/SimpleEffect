/**
 * A/V synchronisation of playback (prompt 13), in headless Chromium with real Web Audio, real
 * WebCodecs and the real compositor, on the « flash + beep » media (`flash-beep.ts`).
 */
import type { Asset, Flicks } from '@se/protocol';
import { afterEach, beforeAll, describe, expect, it } from 'vitest';

import { AudioChunkPool } from '../../src/audio/audio-chunk-pool';
import { audioClipsOf, type PlayableSequence } from '../../src/audio/audio-clips';
import { audioContextClock } from '../../src/audio/audio-clock';
import { createAudioEngineWith, type AudioEngine } from '../../src/audio/audio-engine';
import { AudioGraph } from '../../src/audio/audio-graph';
import { AudioScheduler } from '../../src/audio/audio-scheduler';
import { createMediabunnyAudioSource } from '../../src/audio/audio-source';
import { AudioMeter } from '../../src/audio/meter/audio-meter';
import meterProcessorUrl from '../../src/audio/meter/meter-processor.ts?worker&url';
import { MediabunnyVideoSource } from '../../src/decode/video-source';
import { PlaybackController } from '../../src/playback/playback-controller';
import type { PlaybackBounds } from '../../src/playback/transport';
import type { PreviewEngine } from '../../src/preview/engine';
import { createPreviewEngine } from '../../src/preview/preview-engine';
import { assetTable, clip, item, seconds, videoAsset } from '../../src/testing/document';
import { luminanceAt, readPixels } from '../golden/golden';
import { FLASH_BEEP, flashBeepUrl, onsets } from './flash-beep';

const RATE = { numerator: FLASH_BEEP.fps, denominator: 1 };
const asset: Asset = {
  ...videoAsset({ width: FLASH_BEEP.width, height: FLASH_BEEP.height, frameRate: RATE }),
  decodability: { video: 'decodable', audio: 'decodable' },
};

/** A sequence playing the media's picture on V1 and its sound on A1 from `start`. */
function playable(start: Flicks): PlayableSequence & {
  format: { width: number; height: number };
} {
  const shot = item(start, seconds(FLASH_BEEP.seconds), clip(asset.id));
  const sound = item(start, seconds(FLASH_BEEP.seconds), clip(asset.id));
  const mix = { muted: false, solo: false, volumeDb: 0, pan: 0, hidden: false };
  return {
    format: { width: FLASH_BEEP.width, height: FLASH_BEEP.height },
    tracks: [
      { id: 'v1', kind: 'video', ...mix, items: [shot] },
      { id: 'a1', kind: 'audio', ...mix, items: [sound] },
    ],
  };
}

/** AudioWorklet module posting the context time of every sound onset after 0.3 s of silence. */
const ONSET_DETECTOR = `
registerProcessor('onset-detector', class extends AudioWorkletProcessor {
  quiet = sampleRate;
  process(inputs) {
    const samples = inputs[0]?.[0];
    if (samples === undefined) return true;
    for (let index = 0; index < samples.length; index++) {
      if (Math.abs(samples[index]) > 0.02) {
        if (this.quiet > 0.3 * sampleRate) this.port.postMessage(currentTime + index / sampleRate);
        this.quiet = 0;
      } else this.quiet += 1;
    }
    return true;
  }
});`;

let url = '';
beforeAll(async () => {
  url = await flashBeepUrl();
});

let cleanups: (() => void)[] = [];
afterEach(() => {
  for (const cleanup of cleanups) cleanup();
  cleanups = [];
});

describe('audio scheduling (offline render)', () => {
  for (const start of [0, 600]) {
    it(`starts every beep on its sample, the clip at ${String(start)} s`, async () => {
      const { sampleRate } = FLASH_BEEP;
      const context = new OfflineAudioContext({
        numberOfChannels: 2,
        length: 5 * sampleRate,
        sampleRate,
      });
      const graph = new AudioGraph(context);
      const pool = new AudioChunkPool({ createReader: createMediabunnyAudioSource });
      pool.setLocator(() => url);
      for (let index = 0; index < FLASH_BEEP.seconds; index++) await pool.load(asset.id, index);
      let now = 0;
      const scheduler = new AudioScheduler(
        { currentTime: () => now, play: (request, ended) => graph.play(request, ended) },
        pool,
      );
      const sequence = playable(seconds(start));
      graph.setMix(sequence.tracks);
      scheduler.setClips(audioClipsOf(sequence));
      const bounds: PlaybackBounds = {
        span: { start: 0 as Flicks, end: seconds(start + 4) },
        loop: false,
      };
      scheduler.setTransport({ rate: 1, anchorTime: seconds(start), anchorClock: 0.5 }, bounds);
      for (; now < 5; now += 0.025) scheduler.tick();
      const rendered = await context.startRendering();
      const found = onsets(rendered.getChannelData(0), 0.02);
      const expected = [0, 1, 2, 3].map((second) => (0.5 + second) * sampleRate + 1);
      expect(found).toHaveLength(4);
      found.forEach((onset, index) => {
        expect(Math.abs(onset - (expected[index] ?? 0))).toBeLessThanOrEqual(2);
      });
    });
  }
});

describe('picture and sound in real time', () => {
  // Real time: a busy machine can drop the one-frame flash; a retry measures again.
  it('shows each flash within 45 ms of its beep, and meters the sound', { retry: 2 }, async () => {
    const context = new AudioContext({ latencyHint: 'interactive' });
    await context.resume();
    expect(context.state).toBe('running');
    const tap = context.createGain();
    tap.connect(context.destination);
    const beeps: number[] = [];
    const detectorUrl = URL.createObjectURL(
      new Blob([ONSET_DETECTOR], { type: 'text/javascript' }),
    );
    await context.audioWorklet.addModule(detectorUrl);
    const detector = new AudioWorkletNode(context, 'onset-detector');
    detector.port.onmessage = (event: MessageEvent<number>) => beeps.push(event.data);
    tap.connect(detector);
    const meter = await AudioMeter.create(context, meterProcessorUrl);
    const audio: AudioEngine = createAudioEngineWith({
      context,
      clock: audioContextClock(context),
      meter,
      locator: () => url,
      createReader: createMediabunnyAudioSource,
      destination: tap,
    });
    const canvas = document.createElement('canvas');
    const picture: PreviewEngine = await createPreviewEngine({
      canvas,
      width: FLASH_BEEP.width,
      height: FLASH_BEEP.height,
      preserveDrawingBuffer: true,
      locator: () => url,
      pool: { createVideoSource: (source) => new MediabunnyVideoSource(source, 'prefer-software') },
    });
    cleanups.push(() => {
      picture.destroy();
      audio.destroy();
    });
    const scene = { sequence: playable(0 as Flicks), assets: assetTable(asset) };
    picture.setScene(scene);
    audio.setScene(scene);
    await picture.settled();

    const controller = new PlaybackController({
      clock: audio.clock,
      frameRate: RATE,
      onFrame: (time) => {
        picture.setTime(time);
      },
      onTransportChange: (transport, bounds) => {
        audio.setTransport(transport, bounds);
        picture.setPlayback(transport === null ? null : { rate: transport.rate, frameRate: RATE });
      },
    });
    controller.setContentEnd(seconds(FLASH_BEEP.seconds));

    const flashes: number[] = [];
    let loudest = Number.NEGATIVE_INFINITY;
    let white = true;
    let watching = true;
    const watch = () => {
      if (!watching) return;
      const isWhite =
        luminanceAt(readPixels(canvas), FLASH_BEEP.width / 2, FLASH_BEEP.height / 2) > 128;
      if (isWhite && !white) flashes.push(audio.clock.now());
      white = isWhite;
      loudest = Math.max(loudest, ...audio.levels().peakDb);
      requestAnimationFrame(watch);
    };
    // Playing first: in each refresh the controller draws, then `watch` reads what it drew.
    controller.play();
    requestAnimationFrame(watch);
    await new Promise((resolve) => setTimeout(resolve, 3500));
    controller.pause();
    watching = false;

    // Flashes and beeps at 1, 2 and 3 s (the flash at 0 s is on screen before playing). Each
    // flash is timed when drawn (it reaches the screen at the next refresh, which the controller
    // anticipates); each beep when its first sample is rendered on the context timeline.
    expect(flashes.length).toBeGreaterThanOrEqual(3);
    const offsets = flashes.map((flash) => {
      const beep = beeps.reduce((best, candidate) =>
        Math.abs(candidate - flash) < Math.abs(best - flash) ? candidate : best,
      );
      return flash - beep;
    });
    const measured = offsets.map((offset) => `${(offset * 1000).toFixed(1)} ms`).join(', ');
    for (const offset of offsets) expect(Math.abs(offset), measured).toBeLessThan(0.045);
    expect(loudest).toBeGreaterThan(-12);
    URL.revokeObjectURL(detectorUrl);
  });
});
