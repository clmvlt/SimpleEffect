import type {
  AudioExportSettings,
  ExportContainer,
  ExportQuality,
  VideoExportSettings,
} from '@se/protocol';
import {
  AudioSample,
  AudioSampleSource,
  CanvasSource,
  canEncodeVideo,
  Mp4OutputFormat,
  MovOutputFormat,
  Output,
  Quality,
  StreamTarget,
  WavOutputFormat,
  WebMOutputFormat,
  type OutputFormat,
  type QualityLevel,
  type VideoEncodingConfig,
} from 'mediabunny';

import {
  audioCodecName,
  maximumAudioPackets,
  maximumVideoPackets,
  webCodecsVideoCodec,
} from './export-codecs';
import { sinkWritable, type ExportSink, type ExportStreamName } from './export-sink';
import { EXPORT_CHANNELS } from './export-timing';

/** Bytes per write sent to the core (fewer, larger uploads; below its 64 MiB limit). */
export const UPLOAD_CHUNK_BYTES = 8 * 1024 * 1024;
/** Sound is given to encoders in pieces of this many samples (one second at 48 kHz). */
const AUDIO_PIECE_FRAMES = 48_000;

const QUALITY_LEVELS: Record<ExportQuality, QualityLevel> = {
  low: 'low',
  medium: 'medium',
  high: 'high',
  veryHigh: 'very-high',
};

/** Mediabunny's quality of a picture rate: a level, or an explicit bitrate. */
function videoQuality(video: VideoExportSettings): Quality {
  return video.rate.kind === 'quality'
    ? new Quality(QUALITY_LEVELS[video.rate.level])
    : new Quality({ bitrate: video.rate.bitsPerSecond });
}

/** What a muxed output holds. */
export interface MuxedOutputPlan {
  readonly container: ExportContainer;
  readonly sink: ExportSink;
  readonly stream: ExportStreamName;
  readonly canvas: OffscreenCanvas | null;
  readonly video: VideoExportSettings | null;
  readonly audio: AudioExportSettings | null;
  readonly frameRate: number;
  readonly frames: number;
  readonly seconds: number;
}

/** An open Mediabunny output and its sources. */
export interface MuxedOutput {
  readonly output: Output;
  readonly video: CanvasSource | null;
  readonly audio: AudioSampleSource | null;
}

function formatOf(container: ExportContainer): OutputFormat {
  switch (container) {
    case 'mp4':
    case 'm4a':
      // Room for the index at the start: files play while they download (web uploads).
      return new Mp4OutputFormat({ fastStart: 'reserve' });
    case 'mov':
      return new MovOutputFormat({ fastStart: 'reserve' });
    case 'webm':
      return new WebMOutputFormat();
    case 'wav':
      return new WavOutputFormat({ large: true });
  }
}

/**
 * The encoder configuration of `video`: hardware preferred when this machine's WebCodecs can
 * encode it so (much faster), else whatever encoder it has (software). Rejects when it cannot
 * encode the codec at all.
 */
export async function videoEncodingConfig(
  video: VideoExportSettings,
  frameRate: number,
): Promise<VideoEncodingConfig> {
  const codec = webCodecsVideoCodec(video.codec);
  if (codec === null) throw new Error(`${video.codec} is encoded by ffmpeg, not by the renderer`);
  const quality = videoQuality(video);
  const base = { codec, quality, keyFrameInterval: 2, latencyMode: 'quality' as const };
  const probe = { width: video.width, height: video.height, frameRate, quality };
  if (await canEncodeVideo(codec, { ...probe, hardwareAcceleration: 'prefer-hardware' })) {
    return { ...base, hardwareAcceleration: 'prefer-hardware' };
  }
  if (await canEncodeVideo(codec, probe)) return base;
  throw new Error(
    `this computer cannot encode ${video.codec} at ${String(video.width)}×${String(video.height)}: choose another codec or size`,
  );
}

/** Opens an output writing `plan.stream` of the sink; `start()` it after adding nothing else. */
export async function openMuxedOutput(plan: MuxedOutputPlan): Promise<MuxedOutput> {
  const output = new Output({
    format: formatOf(plan.container),
    target: new StreamTarget(sinkWritable(plan.sink, plan.stream), {
      chunked: true,
      chunkSize: UPLOAD_CHUNK_BYTES,
    }),
  });
  let video: CanvasSource | null = null;
  if (plan.video !== null && plan.canvas !== null) {
    video = new CanvasSource(plan.canvas, await videoEncodingConfig(plan.video, plan.frameRate));
    output.addVideoTrack(video, {
      frameRate: plan.frameRate,
      maximumPacketCount: maximumVideoPackets(plan.frames),
    });
  }
  let audio: AudioSampleSource | null = null;
  if (plan.audio !== null) {
    const codec = audioCodecName(plan.audio.codec);
    audio = new AudioSampleSource(
      codec === 'pcm-s16'
        ? { codec }
        : { codec, quality: new Quality({ bitrate: plan.audio.bitsPerSecond }) },
    );
    output.addAudioTrack(audio, { maximumPacketCount: maximumAudioPackets(plan.seconds) });
  }
  await output.start();
  return { output, video, audio };
}

/** Adds a mixed block of sound starting at file second `seconds`, in one-second pieces. */
export async function addSound(
  source: AudioSampleSource,
  channels: readonly Float32Array[],
  seconds: number,
  sampleRate: number,
): Promise<void> {
  const length = channels[0]?.length ?? 0;
  for (let first = 0; first < length; first += AUDIO_PIECE_FRAMES) {
    const count = Math.min(AUDIO_PIECE_FRAMES, length - first);
    const planar = new Float32Array(count * EXPORT_CHANNELS);
    for (let channel = 0; channel < EXPORT_CHANNELS; channel++) {
      const data = channels[channel] ?? channels[0];
      if (data !== undefined) planar.set(data.subarray(first, first + count), channel * count);
    }
    const sample = new AudioSample({
      data: planar,
      format: 'f32-planar',
      numberOfChannels: EXPORT_CHANNELS,
      sampleRate,
      timestamp: seconds + first / sampleRate,
    });
    try {
      await source.add(sample);
    } finally {
      sample.close();
    }
  }
}
