/**
 * Helpers of the export browser tests: an in-memory export sink, a generated tone (WAV served
 * from a blob URL, no binary fixture) and the decoding of an exported file with Mediabunny.
 */
import type { Asset } from '@se/protocol';
import { ALL_FORMATS, BufferSource, Input, VideoSampleSink } from 'mediabunny';

import type { ExportSink, ExportStreamName } from '../../src/export/export-sink';
import { seconds as flicksOf, videoAsset } from '../../src/testing/document';

/** An export sink keeping every stream in memory (positional writes, like the core). */
export class MemorySink implements ExportSink {
  readonly streams: Record<ExportStreamName, Uint8Array> = {
    data: new Uint8Array(0),
    audio: new Uint8Array(0),
  };
  readonly frames: Uint8Array[] = [];
  readonly thumbnails: Blob[] = [];

  write(stream: ExportStreamName, position: number, bytes: Uint8Array): Promise<void> {
    const current = this.streams[stream];
    const next = new Uint8Array(Math.max(current.length, position + bytes.length));
    next.set(current);
    next.set(bytes, position);
    this.streams[stream] = next;
    return Promise.resolve();
  }

  frame(rgba: Uint8Array): Promise<void> {
    this.frames.push(rgba.slice());
    return Promise.resolve();
  }

  thumbnail(jpeg: Blob): Promise<void> {
    this.thumbnails.push(jpeg);
    return Promise.resolve();
  }
}

/** A mono 16-bit WAV of `seconds` of a `hertz` sine at half scale. */
export function toneWav(seconds: number, hertz: number, sampleRate = 48_000): Blob {
  const frames = Math.round(seconds * sampleRate);
  const bytes = new ArrayBuffer(44 + frames * 2);
  const view = new DataView(bytes);
  const text = (at: number, value: string) => {
    for (let index = 0; index < value.length; index++) {
      view.setUint8(at + index, value.charCodeAt(index));
    }
  };
  text(0, 'RIFF');
  view.setUint32(4, 36 + frames * 2, true);
  text(8, 'WAVE');
  text(12, 'fmt ');
  view.setUint32(16, 16, true);
  view.setUint16(20, 1, true);
  view.setUint16(22, 1, true);
  view.setUint32(24, sampleRate, true);
  view.setUint32(28, sampleRate * 2, true);
  view.setUint16(32, 2, true);
  view.setUint16(34, 16, true);
  text(36, 'data');
  view.setUint32(40, frames * 2, true);
  for (let frame = 0; frame < frames; frame++) {
    const value = 0.5 * Math.sin((2 * Math.PI * hertz * frame) / sampleRate);
    view.setInt16(44 + frame * 2, Math.round(value * 32767), true);
  }
  return new Blob([bytes], { type: 'audio/wav' });
}

/** A probed, decodable sound asset. */
export function soundAsset(seconds: number): Asset {
  const base = videoAsset({ width: 2, height: 2 });
  return {
    ...base,
    kind: 'audio',
    originalPath: '/media/tone.wav',
    mediaInfo: {
      duration: flicksOf(seconds),
      container: 'wav',
      videoStream: null,
      audioStreams: [],
    },
    decodability: { video: 'noStream', audio: 'decodable' },
  };
}

/** A decoded exported file: its frames (as pixels) and its sound's duration. */
export interface DecodedExport {
  readonly frames: ImageData[];
  readonly timestamps: number[];
  readonly videoCodec: string | null;
  readonly audioCodec: string | null;
  readonly audioSeconds: number | null;
}

/** Decodes every frame of an exported file and reads its sound track's duration. */
export async function decodeExport(bytes: Uint8Array): Promise<DecodedExport> {
  const input = new Input({ source: new BufferSource(bytes), formats: ALL_FORMATS });
  const video = await input.getPrimaryVideoTrack();
  const audio = await input.getPrimaryAudioTrack();
  const frames: ImageData[] = [];
  const timestamps: number[] = [];
  if (video !== null) {
    const canvas = new OffscreenCanvas(
      await video.getDisplayWidth(),
      await video.getDisplayHeight(),
    );
    const context = canvas.getContext('2d');
    if (context === null) throw new Error('no 2D context');
    for await (const sample of new VideoSampleSink(video).samples()) {
      sample.draw(context, 0, 0);
      timestamps.push(sample.timestamp);
      sample.close();
      frames.push(context.getImageData(0, 0, canvas.width, canvas.height));
    }
  }
  return {
    frames,
    timestamps,
    videoCodec: video === null ? null : await video.getCodec(),
    audioCodec: audio === null ? null : await audio.getCodec(),
    audioSeconds: audio === null ? null : await audio.computeDuration(),
  };
}
