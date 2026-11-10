import type { ExportPipeline, ExportSettings, Flicks, FrameRate } from '@se/protocol';
import type { AudioSampleSource } from 'mediabunny';

import { addSound, openMuxedOutput, type MuxedOutput } from './export-output';
import type { ExportSink } from './export-sink';
import {
  EXPORT_SAMPLE_RATE,
  audioBlocks,
  exportFrames,
  type AudioBlock,
  type ExportFrames,
} from './export-timing';

/** What to render: the span, its settings and how the file is put together. */
export interface ExportRenderJob {
  readonly settings: ExportSettings;
  readonly pipeline: ExportPipeline;
  readonly start: Flicks;
  readonly end: Flicks;
  readonly frameRate: FrameRate;
}

/** Where frames are drawn, where bytes go and where mixed sound comes from. */
export interface ExportRenderContext {
  /** The canvas frames are drawn on (`null` for sound-only exports). */
  readonly canvas: OffscreenCanvas | null;
  /** Draws the frame at a sequence time on `canvas`. */
  readonly draw: (time: Flicks) => Promise<void>;
  /** Reads `canvas`'s pixels as RGBA (masters only). */
  readonly readRgba: () => Uint8Array;
  readonly sink: ExportSink;
  /** Mixed sound of block `index` (one buffer per channel). */
  readonly sound: (index: number) => Promise<Float32Array[]>;
  /** Sends a thumbnail of the frame on `canvas` (failures ignored). */
  readonly thumbnail: () => Promise<void>;
  readonly onProgress: (done: number, total: number) => void;
  readonly signal: AbortSignal;
}

/** Counts finished units (frames, sound blocks written before them). */
class Progress {
  private done = 0;
  constructor(
    private readonly total: number,
    private readonly report: (done: number, total: number) => void,
  ) {}

  step(): void {
    this.done += 1;
    this.report(this.done, this.total);
  }
}

/** Adds mixed blocks to an encoder in order, the next block mixing while one is encoded. */
class SoundFeeder {
  private next = 0;
  private pending: Promise<Float32Array[]> | null = null;

  constructor(
    private readonly source: AudioSampleSource,
    private readonly blocks: readonly AudioBlock[],
    private readonly mix: (index: number) => Promise<Float32Array[]>,
    private readonly onBlock: () => void = () => undefined,
  ) {}

  /** Adds every block starting before file second `seconds`. */
  async through(seconds: number, signal: AbortSignal): Promise<void> {
    for (let block = this.blocks[this.next]; block !== undefined && block.seconds < seconds;) {
      signal.throwIfAborted();
      const channels = await (this.pending ?? this.mix(block.index));
      this.next += 1;
      const following = this.blocks[this.next];
      this.pending = following === undefined ? null : this.mix(following.index);
      // A failure surfaces when the block is awaited; never as an unhandled rejection meanwhile.
      this.pending?.catch(() => undefined);
      await addSound(this.source, channels, block.seconds, EXPORT_SAMPLE_RATE);
      this.onBlock();
      block = following;
    }
  }
}

/** The frame whose picture becomes the thumbnail (a tenth in, past fades from black). */
function thumbnailIndex(frames: ExportFrames): number {
  return Math.min(frames.count - 1, Math.floor(frames.count / 10));
}

/**
 * Renders an export (ADR-0038): frames drawn in order and sound mixed in blocks, written the
 * way the core's pipeline asks — one muxed file, RGBA frames for ffmpeg after the WAV of the
 * sound (masters), or a muxed picture plus a WAV that ffmpeg encodes as AAC. Rejects on the first
 * failure or when `signal` aborts; the caller tells the core.
 */
export async function renderExport(
  job: ExportRenderJob,
  context: ExportRenderContext,
): Promise<void> {
  const frames =
    job.settings.video === null ? null : exportFrames(job.start, job.end, job.frameRate);
  const blocks = job.settings.audio === null ? [] : audioBlocks(job.start, job.end);
  switch (job.pipeline.kind) {
    case 'muxed':
      return renderMuxed(job, context, frames, blocks);
    case 'ffmpegFrames': {
      const soundFirst = job.pipeline.withAudio ? blocks : [];
      const progress = new Progress(soundFirst.length + (frames?.count ?? 0), context.onProgress);
      if (soundFirst.length > 0) await writeWav(job, context, soundFirst, progress);
      return renderRawFrames(context, requireFrames(frames), progress);
    }
    case 'aacRemux': {
      const progress = new Progress(blocks.length + (frames?.count ?? 0), context.onProgress);
      await writeWav(job, context, blocks, progress);
      if (!job.pipeline.withVideo) return;
      return renderMuxed(
        { ...job, settings: { ...job.settings, audio: null } },
        context,
        frames,
        [],
        progress,
      );
    }
  }
}

function requireFrames(frames: ExportFrames | null): ExportFrames {
  if (frames === null) throw new Error('this export has no picture to encode');
  return frames;
}

/** One muxed file: the picture frame by frame with the sound added just ahead of it. */
async function renderMuxed(
  job: ExportRenderJob,
  context: ExportRenderContext,
  frames: ExportFrames | null,
  blocks: readonly AudioBlock[],
  shared?: Progress,
): Promise<void> {
  const seconds = blocks.reduce((sum, block) => sum + block.frames, 0) / EXPORT_SAMPLE_RATE || 0;
  const progress =
    shared ?? new Progress(frames === null ? blocks.length : frames.count, context.onProgress);
  const muxed = await openMuxedOutput({
    container: job.settings.container,
    sink: context.sink,
    stream: 'data',
    canvas: context.canvas,
    video: job.settings.video,
    audio: blocks.length > 0 ? job.settings.audio : null,
    frameRate: job.frameRate.numerator / job.frameRate.denominator,
    frames: frames?.count ?? 0,
    seconds: frames === null ? seconds : Math.max(seconds, frames.count * frames.frameSeconds),
  });
  await completing(muxed, async () => {
    const feeder =
      muxed.audio === null
        ? null
        : new SoundFeeder(
            muxed.audio,
            blocks,
            context.sound,
            frames === null
              ? () => {
                  progress.step();
                }
              : undefined,
          );
    if (frames !== null && muxed.video !== null) {
      for (let index = 0; index < frames.count; index++) {
        context.signal.throwIfAborted();
        await feeder?.through(frames.secondsOf(index) + 1, context.signal);
        await context.draw(frames.timeOf(index));
        await muxed.video.add(frames.secondsOf(index), frames.frameSeconds);
        if (index === thumbnailIndex(frames)) await context.thumbnail().catch(() => undefined);
        progress.step();
      }
    }
    await feeder?.through(Number.POSITIVE_INFINITY, context.signal);
  });
}

/** Runs `write`, then closes the sources and finalizes the file; cancels the output on failure. */
async function completing(muxed: MuxedOutput, write: () => Promise<void>): Promise<void> {
  try {
    await write();
    muxed.video?.close();
    muxed.audio?.close();
    await muxed.output.finalize();
  } catch (error) {
    await muxed.output.cancel().catch(() => undefined);
    throw error;
  }
}

/** The sound as a 16-bit WAV on the `audio` stream (ffmpeg's input). */
async function writeWav(
  job: ExportRenderJob,
  context: ExportRenderContext,
  blocks: readonly AudioBlock[],
  progress: Progress,
): Promise<void> {
  const muxed = await openMuxedOutput({
    container: 'wav',
    sink: context.sink,
    stream: 'audio',
    canvas: null,
    video: null,
    audio: { codec: 'pcm', bitsPerSecond: 0, encoder: 'webCodecs' },
    frameRate: job.frameRate.numerator / job.frameRate.denominator,
    frames: 0,
    seconds: blocks.reduce((sum, block) => sum + block.frames, 0) / EXPORT_SAMPLE_RATE,
  });
  await completing(muxed, async () => {
    if (muxed.audio === null) return;
    await new SoundFeeder(muxed.audio, blocks, context.sound, () => {
      progress.step();
    }).through(Number.POSITIVE_INFINITY, context.signal);
  });
}

/** RGBA frames for ffmpeg, one upload under way while the next frame is drawn. */
async function renderRawFrames(
  context: ExportRenderContext,
  frames: ExportFrames,
  progress: Progress,
): Promise<void> {
  let upload: Promise<void> = Promise.resolve();
  for (let index = 0; index < frames.count; index++) {
    context.signal.throwIfAborted();
    await context.draw(frames.timeOf(index));
    const rgba = context.readRgba();
    if (index === thumbnailIndex(frames)) await context.thumbnail().catch(() => undefined);
    await upload;
    upload = context.sink.frame(rgba);
    // Awaited at the next frame or after the loop; never an unhandled rejection meanwhile.
    upload.catch(() => undefined);
    progress.step();
  }
  await upload;
}
