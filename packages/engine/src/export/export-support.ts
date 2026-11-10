import type { AudioExportSettings, VideoExportSettings } from '@se/protocol';
import { canEncodeAudio, canEncodeVideo, Quality } from 'mediabunny';

import { audioCodecName, webCodecsVideoCodec } from './export-codecs';
import { videoQuality } from './export-output';
import { EXPORT_CHANNELS, EXPORT_SAMPLE_RATE } from './export-timing';

/**
 * Who can encode an export's picture here: a hardware or software WebCodecs encoder, ffmpeg (the
 * ProRes and DNxHR masters), or nobody.
 */
export type VideoEncoderSupport = 'hardware' | 'software' | 'ffmpeg' | 'none';

/** Who can encode an export's sound here: WebCodecs (PCM needs no encoder), or nobody. */
export type AudioEncoderSupport = 'webCodecs' | 'none';

/**
 * Which encoders this machine offers for a picture at `frameRate` (`null`: no picture). Probed
 * the way the export Worker configures them (`videoEncodingConfig`), hardware first.
 */
export async function videoEncoderSupport(
  video: VideoExportSettings | null,
  frameRate: number,
): Promise<VideoEncoderSupport | null> {
  if (video === null) return null;
  const codec = webCodecsVideoCodec(video.codec);
  if (codec === null) return 'ffmpeg';
  if (typeof VideoEncoder === 'undefined') return 'none';
  const probe = {
    width: video.width,
    height: video.height,
    frameRate,
    quality: videoQuality(video),
  };
  if (await canEncodeVideo(codec, { ...probe, hardwareAcceleration: 'prefer-hardware' })) {
    return 'hardware';
  }
  return (await canEncodeVideo(codec, probe)) ? 'software' : 'none';
}

/**
 * Whether WebCodecs encodes this sound here (`null`: no sound). AAC is missing from some
 * Chromium builds (Linux): the app then lets ffmpeg encode it (`encoder: 'ffmpeg'`).
 */
export async function audioEncoderSupport(
  audio: AudioExportSettings | null,
): Promise<AudioEncoderSupport | null> {
  if (audio === null) return null;
  const codec = audioCodecName(audio.codec);
  if (codec === 'pcm-s16') return 'webCodecs';
  if (typeof AudioEncoder === 'undefined') return 'none';
  const supported = await canEncodeAudio(codec, {
    numberOfChannels: EXPORT_CHANNELS,
    sampleRate: EXPORT_SAMPLE_RATE,
    quality: new Quality({ bitrate: audio.bitsPerSecond }),
  });
  return supported ? 'webCodecs' : 'none';
}
