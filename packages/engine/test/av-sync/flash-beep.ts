/**
 * The « flash + beep » test media of the A/V sync tests, generated in the browser with Mediabunny
 * (no binary fixture, no ffmpeg): a 25 fps VP8 picture, black except one white frame at each whole
 * second, and a 48 kHz PCM sound, silent except a 1 kHz beep starting exactly at each whole second.
 */
import {
  AudioBufferSource,
  BufferTarget,
  CanvasSource,
  MkvOutputFormat,
  Output,
  Quality,
} from 'mediabunny';
import { commands } from 'vitest/browser';

/** Facts of the generated media. */
export const FLASH_BEEP = {
  width: 320,
  height: 180,
  fps: 25,
  sampleRate: 48_000,
  seconds: 4,
  beepSeconds: 0.04,
  beepHz: 1000,
} as const;

/** Where the test writes it (package-relative, gitignored) and the URL Vite serves it at. */
const FILE = './test/av-sync/generated/flash-beep.mkv';
const URL_PATH = '/test/av-sync/generated/flash-beep.mkv';

function beepBuffer(): AudioBuffer {
  const { sampleRate, seconds, beepSeconds, beepHz } = FLASH_BEEP;
  const buffer = new AudioBuffer({ numberOfChannels: 1, length: sampleRate * seconds, sampleRate });
  const samples = buffer.getChannelData(0);
  for (let second = 0; second < seconds; second++) {
    const start = second * sampleRate;
    for (let index = 0; index < beepSeconds * sampleRate; index++) {
      samples[start + index] = 0.5 * Math.sin((2 * Math.PI * beepHz * index) / sampleRate);
    }
  }
  return buffer;
}

async function encode(): Promise<ArrayBuffer> {
  const { width, height, fps, seconds } = FLASH_BEEP;
  const canvas = new OffscreenCanvas(width, height);
  const context = canvas.getContext('2d');
  if (context === null) throw new Error('no 2D context');
  const output = new Output({ format: new MkvOutputFormat(), target: new BufferTarget() });
  const video = new CanvasSource(canvas, {
    codec: 'vp8',
    quality: new Quality('high'),
    keyFrameInterval: 1,
  });
  const audio = new AudioBufferSource({ codec: 'pcm-f32' });
  output.addVideoTrack(video, { frameRate: fps });
  output.addAudioTrack(audio);
  await output.start();
  for (let frame = 0; frame < fps * seconds; frame++) {
    context.fillStyle = frame % fps === 0 ? 'white' : 'black';
    context.fillRect(0, 0, width, height);
    await video.add(frame / fps, 1 / fps);
  }
  await audio.add(beepBuffer());
  await output.finalize();
  if (output.target.buffer === null) throw new Error('the media was not written');
  return output.target.buffer;
}

function toBase64(bytes: ArrayBuffer): string {
  let binary = '';
  const view = new Uint8Array(bytes);
  for (let index = 0; index < view.length; index += 0x8000) {
    binary += String.fromCharCode(...view.subarray(index, index + 0x8000));
  }
  return btoa(binary);
}

/** Generates the media, writes it where Vite serves it, and returns its URL. */
export async function flashBeepUrl(): Promise<string> {
  await commands.writeFile(FILE, toBase64(await encode()), 'base64');
  return URL_PATH;
}

/**
 * Sample indexes where sound starts after at least `silence` quiet samples (beep onsets), in a
 * rendered channel.
 */
export function onsets(samples: Float32Array, threshold = 0.05, silence = 2400): number[] {
  const found: number[] = [];
  let quiet = silence;
  for (let index = 0; index < samples.length; index++) {
    if (Math.abs(samples[index] ?? 0) > threshold) {
      if (quiet >= silence) found.push(index);
      quiet = 0;
    } else {
      quiet += 1;
    }
  }
  return found;
}
