import type { AssetKind, MediaInfo, StreamDecodability, VideoStreamInfo } from '@se/protocol';

import { audioCodecString, videoCodecString } from './codec-strings';

/** The WebCodecs entry points used to check support (the globals in the renderer; fakes in tests). */
export interface CodecSupport {
  readonly VideoDecoder?: {
    isConfigSupported(config: VideoDecoderConfig): Promise<VideoDecoderSupport>;
  };
  readonly AudioDecoder?: {
    isConfigSupported(config: AudioDecoderConfig): Promise<AudioDecoderSupport>;
  };
  readonly ImageDecoder?: { isTypeSupported(type: string): Promise<boolean> };
}

/** Whether each stream of an asset can be decoded on this machine. */
export interface DecodabilityResult {
  readonly video: StreamDecodability;
  readonly audio: StreamDecodability;
}

/** Still-picture codecs and the MIME type `ImageDecoder` knows them by. */
const IMAGE_TYPES: Readonly<Record<string, string>> = {
  png: 'image/png',
  mjpeg: 'image/jpeg',
  webp: 'image/webp',
  gif: 'image/gif',
  bmp: 'image/bmp',
};

/**
 * Checks with WebCodecs whether this machine decodes the streams of an asset (sent to se-core as
 * `reportDecodability`, `docs/architecture.md` §7). Never rejects: a codec the browser refuses or
 * an API it lacks counts as `undecodable` (the asset then needs a mezzanine).
 */
export async function checkDecodability(
  kind: AssetKind,
  info: MediaInfo,
  support: CodecSupport = globalThis,
): Promise<DecodabilityResult> {
  const [video, audio] = await Promise.all([
    info.videoStream === null
      ? Promise.resolve<StreamDecodability>('noStream')
      : kind === 'image'
        ? checkImage(info.videoStream, support)
        : checkVideo(info.videoStream, support),
    checkAudio(info, support),
  ]);
  return { video, audio };
}

async function checkVideo(
  stream: VideoStreamInfo,
  support: CodecSupport,
): Promise<StreamDecodability> {
  const codec = videoCodecString(stream);
  if (codec === null || support.VideoDecoder === undefined) {
    return 'undecodable';
  }
  const config: VideoDecoderConfig = {
    codec,
    codedWidth: stream.width,
    codedHeight: stream.height,
  };
  return verdict(() => support.VideoDecoder?.isConfigSupported(config));
}

async function checkAudio(info: MediaInfo, support: CodecSupport): Promise<StreamDecodability> {
  const stream = info.audioStreams[0];
  if (stream === undefined) {
    return 'noStream';
  }
  const codec = audioCodecString(stream);
  if (codec === null || support.AudioDecoder === undefined) {
    return 'undecodable';
  }
  const config: AudioDecoderConfig = {
    codec,
    sampleRate: stream.sampleRate,
    numberOfChannels: stream.channels,
  };
  return verdict(() => support.AudioDecoder?.isConfigSupported(config));
}

async function checkImage(
  stream: VideoStreamInfo,
  support: CodecSupport,
): Promise<StreamDecodability> {
  const type = IMAGE_TYPES[stream.codec];
  if (type === undefined) {
    return 'undecodable';
  }
  if (support.ImageDecoder === undefined) {
    // Without ImageDecoder the browser still draws these formats through <img>/createImageBitmap.
    return 'decodable';
  }
  try {
    return (await support.ImageDecoder.isTypeSupported(type)) ? 'decodable' : 'undecodable';
  } catch {
    return 'undecodable';
  }
}

/** `decodable` when the support check says so; refusals and thrown errors are `undecodable`. */
async function verdict(
  check: () => Promise<{ supported?: boolean | undefined }> | undefined,
): Promise<StreamDecodability> {
  try {
    const result = await check();
    return result?.supported === true ? 'decodable' : 'undecodable';
  } catch {
    return 'undecodable';
  }
}
