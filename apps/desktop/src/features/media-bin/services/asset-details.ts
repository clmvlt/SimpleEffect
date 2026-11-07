import type { Asset, AudioStreamInfo, VideoStreamInfo } from '@se/protocol';

/** Readable names of common codecs (others are shown as ffprobe names them, upper-cased). */
const CODEC_NAMES: Readonly<Record<string, string>> = {
  h264: 'H.264',
  hevc: 'HEVC (H.265)',
  vp8: 'VP8',
  vp9: 'VP9',
  av1: 'AV1',
  prores: 'Apple ProRes',
  dnxhd: 'Avid DNxHD/HR',
  mpeg2video: 'MPEG-2',
  mjpeg: 'Motion JPEG',
  png: 'PNG',
  webp: 'WebP',
  aac: 'AAC',
  mp3: 'MP3',
  opus: 'Opus',
  vorbis: 'Vorbis',
  flac: 'FLAC',
  ac3: 'Dolby Digital',
  eac3: 'Dolby Digital Plus',
};

/** `H.264 High`, `Apple ProRes HQ`, `PCM S24LE`… */
export function codecLabel(codec: string, profile: string | null): string {
  const name =
    CODEC_NAMES[codec] ??
    (codec.startsWith('pcm_') ? `PCM ${codec.slice(4).toUpperCase()}` : codec.toUpperCase());
  return profile === null || profile === '' ? name : `${name} ${profile}`;
}

/** Width and height as shown (rotation applied): portrait phone footage is taller than wide. */
export function displaySize(stream: VideoStreamInfo): { width: number; height: number } {
  return stream.rotation % 180 === 90
    ? { width: stream.height, height: stream.width }
    : { width: stream.width, height: stream.height };
}

/** Frames per second with at most 3 decimals (`29.97`, `25`), or `null` for variable rates. */
export function framesPerSecond(stream: VideoStreamInfo): string | null {
  const rate = stream.frameRate;
  if (rate === null) {
    return null;
  }
  return String(Math.round((rate.numerator / rate.denominator) * 1000) / 1000);
}

/** Sample rate in kHz with at most 1 decimal (`48`, `44.1`). */
export function kilohertz(stream: AudioStreamInfo): string {
  return String(Math.round(stream.sampleRate / 100) / 10);
}

/** Whether the asset is a video whose picture WebCodecs cannot decode directly. */
export function needsMezzanine(asset: Asset): boolean {
  return asset.decodability.video === 'undecodable';
}
