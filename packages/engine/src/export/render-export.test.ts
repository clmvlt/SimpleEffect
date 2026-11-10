import type { ExportSettings, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds } from '../testing/document';
import type { ExportSink, ExportStreamName } from './export-sink';
import { audioBlocks } from './export-timing';
import { renderExport, type ExportRenderContext, type ExportRenderJob } from './render-export';

const RATE = { numerator: 25, denominator: 1 };

/** An in-memory sink: positional writes per stream, frames and thumbnails in order. */
class MemorySink implements ExportSink {
  readonly streams: Record<ExportStreamName, Uint8Array> = {
    data: new Uint8Array(0),
    audio: new Uint8Array(0),
  };
  readonly frames: Uint8Array[] = [];
  thumbnails = 0;

  write(stream: ExportStreamName, position: number, bytes: Uint8Array): Promise<void> {
    const current = this.streams[stream];
    const next = new Uint8Array(Math.max(current.length, position + bytes.length));
    next.set(current);
    next.set(bytes, position);
    this.streams[stream] = next;
    return Promise.resolve();
  }

  frame(rgba: Uint8Array): Promise<void> {
    this.frames.push(rgba);
    return Promise.resolve();
  }

  thumbnail(): Promise<void> {
    this.thumbnails += 1;
    return Promise.resolve();
  }
}

/** A sine of `hertz` over the blocks of `[start, end)`, continuing across blocks. */
function sineBlocks(start: Flicks, end: Flicks, hertz = 441) {
  const blocks = audioBlocks(start, end);
  return (index: number): Promise<Float32Array[]> => {
    const block = blocks[index];
    if (block === undefined) return Promise.reject(new Error(`no block ${String(index)}`));
    const samples = new Float32Array(block.frames);
    const first = block.seconds * 48_000;
    for (let at = 0; at < block.frames; at++) {
      samples[at] = 0.5 * Math.sin((2 * Math.PI * hertz * (first + at)) / 48_000);
    }
    return Promise.resolve([samples, samples.slice()]);
  };
}

function context(sink: MemorySink, overrides: Partial<ExportRenderContext> = {}) {
  const drawn: Flicks[] = [];
  const progress: [number, number][] = [];
  const value: ExportRenderContext = {
    canvas: null,
    draw: (time) => {
      drawn.push(time);
      return Promise.resolve();
    },
    readRgba: () => new Uint8Array([drawn.length, 0, 0, 255]),
    sink,
    sound: sineBlocks(0 as Flicks, seconds(1)),
    thumbnail: () => sink.thumbnail(),
    onProgress: (done, total) => progress.push([done, total]),
    signal: new AbortController().signal,
    ...overrides,
  };
  return { value, drawn, progress };
}

const master: ExportSettings = {
  container: 'mov',
  video: {
    codec: 'prores422Hq',
    width: 2,
    height: 2,
    rate: { kind: 'quality', level: 'veryHigh' },
  },
  audio: { codec: 'pcm', bitsPerSecond: 0, encoder: 'webCodecs' },
};

/** The 16-bit samples of a WAV's first channel (after its header's `data` chunk). */
function wavSamples(bytes: Uint8Array): { channels: number; rate: number; left: Int16Array } {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const text = (at: number) => String.fromCharCode(...bytes.subarray(at, at + 4));
  let at = 12;
  let channels = 0;
  let rate = 0;
  while (at < bytes.length) {
    const size = view.getUint32(at + 4, true);
    if (text(at) === 'fmt ') {
      channels = view.getUint16(at + 10, true);
      rate = view.getUint32(at + 12, true);
    }
    if (text(at) === 'data') {
      const frames = Math.min(size, bytes.length - at - 8) / 2 / channels;
      const left = new Int16Array(frames);
      for (let frame = 0; frame < frames; frame++) {
        left[frame] = view.getInt16(at + 8 + frame * channels * 2, true);
      }
      return { channels, rate, left };
    }
    at += 8 + size + (size % 2);
  }
  throw new Error('no data chunk');
}

describe('renderExport', () => {
  it('writes the WAV of a master first, then one RGBA frame per sequence frame', async () => {
    const sink = new MemorySink();
    const { value, drawn, progress } = context(sink, {
      sound: sineBlocks(seconds(10), seconds(12)),
    });
    const job: ExportRenderJob = {
      settings: master,
      pipeline: { kind: 'ffmpegFrames', withAudio: true },
      start: seconds(10),
      end: seconds(12),
      frameRate: RATE,
    };
    await renderExport(job, value);
    expect(drawn).toHaveLength(50);
    expect(drawn[0]).toBe(seconds(10));
    expect(drawn[25]).toBe(seconds(11));
    expect(sink.frames).toHaveLength(50);
    expect(sink.frames[3]?.[0]).toBe(4);
    expect(sink.thumbnails).toBe(1);
    expect(sink.streams.data).toHaveLength(0);
    const wav = wavSamples(sink.streams.audio);
    expect(wav.channels).toBe(2);
    expect(wav.rate).toBe(48_000);
    expect(wav.left).toHaveLength(96_000);
    // One sound block, then 50 frames.
    expect(progress.at(-1)).toEqual([51, 51]);
  });

  it('joins sound blocks without a step at their boundaries', async () => {
    const sink = new MemorySink();
    const { value } = context(sink);
    await renderExport(
      {
        settings: { container: 'm4a', video: null, audio: { ...aac } },
        pipeline: { kind: 'aacRemux', withVideo: false },
        start: 0 as Flicks,
        end: seconds(65),
        frameRate: RATE,
      },
      { ...value, sound: sineBlocks(0 as Flicks, seconds(65)) },
    );
    const { left } = wavSamples(sink.streams.audio);
    // Three blocks (30 s, 30 s, 5 s), exactly 65 s of samples.
    expect(left).toHaveLength(65 * 48_000);
    let largestStep = 0;
    for (let at = 1; at < left.length; at++) {
      largestStep = Math.max(largestStep, Math.abs((left[at] ?? 0) - (left[at - 1] ?? 0)));
    }
    // A 441 Hz sine at half scale moves at most ~950 per sample; a seam would jump further.
    expect(largestStep).toBeLessThan(1000);
  });

  it('stops at the first frame after the signal aborts', async () => {
    const sink = new MemorySink();
    const controller = new AbortController();
    const { value, drawn } = context(sink, {
      signal: controller.signal,
      draw: () => {
        if (sink.frames.length === 4) controller.abort();
        return Promise.resolve();
      },
    });
    const job: ExportRenderJob = {
      settings: { ...master, audio: null },
      pipeline: { kind: 'ffmpegFrames', withAudio: false },
      start: 0 as Flicks,
      end: seconds(2),
      frameRate: RATE,
    };
    await expect(renderExport(job, value)).rejects.toThrow();
    expect(drawn).toHaveLength(0);
    expect(sink.frames.length).toBeLessThanOrEqual(5);
  });
});

const aac = { codec: 'aac' as const, bitsPerSecond: 192_000, encoder: 'ffmpeg' as const };
