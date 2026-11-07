import type { AudioStreamInfo, VideoStreamInfo } from '@se/protocol';

/**
 * WebCodecs codec strings (RFC 6381 style) built from probed stream facts, for
 * `VideoDecoder.isConfigSupported` / `AudioDecoder.isConfigSupported`. Codecs WebCodecs cannot
 * decode at all (ProRes, DNxHR, MPEG-2…) give `null`.
 */

/** `avc1.PPCCLL`: profile_idc and constraint flags of each H.264 profile ffprobe names. */
const AVC_PROFILES: Readonly<Record<string, readonly [number, number]>> = {
  Baseline: [0x42, 0x00],
  'Constrained Baseline': [0x42, 0x40],
  Main: [0x4d, 0x00],
  Extended: [0x58, 0x00],
  High: [0x64, 0x00],
  'High 10': [0x6e, 0x00],
  'High 10 Intra': [0x6e, 0x10],
  'High 4:2:2': [0x7a, 0x00],
  'High 4:2:2 Intra': [0x7a, 0x10],
  'High 4:4:4 Predictive': [0xf4, 0x00],
  'High 4:4:4 Intra': [0xf4, 0x10],
  'CAVLC 4:4:4': [0x2c, 0x10],
};

const VP9_LEVELS = new Set([10, 11, 20, 21, 30, 31, 40, 41, 50, 51, 52, 60, 61, 62]);

/** PCM sample formats WebCodecs names (little-endian only). */
const PCM_CODECS: Readonly<Record<string, string>> = {
  pcm_u8: 'pcm-u8',
  pcm_s16le: 'pcm-s16',
  pcm_s24le: 'pcm-s24',
  pcm_s32le: 'pcm-s32',
  pcm_f32le: 'pcm-f32',
  pcm_mulaw: 'ulaw',
  pcm_alaw: 'alaw',
};

const AUDIO_CODECS: Readonly<Record<string, string>> = {
  mp3: 'mp3',
  opus: 'opus',
  vorbis: 'vorbis',
  flac: 'flac',
  ac3: 'ac-3',
  eac3: 'ec-3',
  ...PCM_CODECS,
};

/** The codec string of a picture stream, or `null` when WebCodecs has no decoder for it. */
export function videoCodecString(stream: VideoStreamInfo): string | null {
  switch (stream.codec) {
    case 'h264':
      return avcCodecString(stream);
    case 'hevc':
      return hevcCodecString(stream);
    case 'vp9':
      return vp9CodecString(stream);
    case 'av1':
      return av1CodecString(stream);
    case 'vp8':
      return 'vp8';
    default:
      return null;
  }
}

/** The codec string of a sound stream, or `null` when WebCodecs has no decoder for it. */
export function audioCodecString(stream: AudioStreamInfo): string | null {
  if (stream.codec === 'aac') {
    return `mp4a.40.${String(aacObjectType(stream.profile))}`;
  }
  return AUDIO_CODECS[stream.codec] ?? null;
}

function avcCodecString(stream: VideoStreamInfo): string {
  const [profile, constraints] =
    AVC_PROFILES[stream.profile ?? ''] ??
    (stream.bitDepth !== null && stream.bitDepth > 8 ? [0x6e, 0x00] : [0x64, 0x00]);
  const level = stream.level ?? byResolution(stream, [0x2a, 0x33, 0x3c]);
  return `avc1.${hex(profile)}${hex(constraints)}${hex(level)}`;
}

function hevcCodecString(stream: VideoStreamInfo): string {
  const prefix = stream.codecTag === 'hev1' ? 'hev1' : 'hvc1';
  const profile = stream.profile ?? '';
  const [profileIdc, compatibility] = profile.startsWith('Main 10')
    ? [2, '4']
    : profile.startsWith('Main Still')
      ? [3, 'E']
      : profile === 'Main'
        ? [1, '6']
        : [4, '10'];
  const level = stream.level ?? byResolution(stream, [123, 153, 183]);
  return `${prefix}.${String(profileIdc)}.${compatibility}.L${String(level)}.B0`;
}

function vp9CodecString(stream: VideoStreamInfo): string {
  const profile = Number(/\d/.exec(stream.profile ?? '')?.[0] ?? 0);
  const level =
    stream.level !== null && VP9_LEVELS.has(stream.level)
      ? stream.level
      : byResolution(stream, [41, 51, 61]);
  return `vp09.${pad(profile)}.${pad(level)}.${pad(stream.bitDepth ?? 8)}`;
}

function av1CodecString(stream: VideoStreamInfo): string {
  const profile = stream.profile === 'High' ? 1 : stream.profile === 'Professional' ? 2 : 0;
  const level = stream.level ?? byResolution(stream, [8, 12, 16]);
  return `av01.${String(profile)}.${pad(level)}M.${pad(stream.bitDepth ?? 8)}`;
}

/** MPEG-4 audio object type of an AAC profile. */
function aacObjectType(profile: string | null): number {
  switch (profile) {
    case 'Main':
      return 1;
    case 'HE-AAC':
      return 5;
    case 'HE-AACv2':
      return 29;
    default:
      return 2;
  }
}

/** A default level when the prober gave none: [up to 1080p, up to 4K, larger]. */
function byResolution(stream: VideoStreamInfo, levels: readonly [number, number, number]): number {
  const pixels = stream.width * stream.height;
  if (pixels <= 1920 * 1088) return levels[0];
  if (pixels <= 4096 * 2304) return levels[1];
  return levels[2];
}

function hex(value: number): string {
  return value.toString(16).toUpperCase().padStart(2, '0');
}

function pad(value: number): string {
  return String(value).padStart(2, '0');
}
