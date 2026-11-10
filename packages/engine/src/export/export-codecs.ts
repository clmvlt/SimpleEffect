import type { AudioExportCodec, VideoExportCodec } from '@se/protocol';

/** A picture codec WebCodecs encodes (Mediabunny's names). */
export type WebCodecsVideoCodec = 'avc' | 'hevc' | 'av1' | 'vp9';
/** A sound codec of exports (Mediabunny's names; PCM is written by Mediabunny itself). */
export type ExportAudioCodecName = 'aac' | 'opus' | 'pcm-s16';

/**
 * The WebCodecs codec of a picture codec, or `null` for the masters ffmpeg encodes from frames
 * (ProRes, DNxHR).
 */
export function webCodecsVideoCodec(codec: VideoExportCodec): WebCodecsVideoCodec | null {
  switch (codec) {
    case 'h264':
      return 'avc';
    case 'hevc':
      return 'hevc';
    case 'av1':
      return 'av1';
    case 'vp9':
      return 'vp9';
    case 'prores422Hq':
    case 'prores4444':
    case 'dnxhrHq':
      return null;
  }
}

/** Mediabunny's name of a sound codec. */
export function audioCodecName(codec: AudioExportCodec): ExportAudioCodecName {
  switch (codec) {
    case 'aac':
      return 'aac';
    case 'opus':
      return 'opus';
    case 'pcm':
      return 'pcm-s16';
  }
}

/** Seconds of sound per packet assumed when reserving room for an MP4 index (10 ms, generous). */
const AUDIO_PACKET_SECONDS = 0.01;
/** Margin on reserved packet counts (Mediabunny refuses packets beyond them). */
const PACKET_MARGIN = 4 / 3;

/** Most packets a picture track of `frames` frames gets (one per frame, with margin). */
export function maximumVideoPackets(frames: number): number {
  return Math.ceil(frames * PACKET_MARGIN) + 16;
}

/** Most packets a sound track of `seconds` gets (one per 10 ms, with margin). */
export function maximumAudioPackets(seconds: number): number {
  return Math.ceil((seconds / AUDIO_PACKET_SECONDS) * PACKET_MARGIN) + 16;
}
