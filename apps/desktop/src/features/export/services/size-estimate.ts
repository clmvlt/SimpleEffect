import type {
  ExportQuality,
  ExportSettings,
  VideoExportCodec,
  VideoExportSettings,
} from '@se/protocol';

/** Quality levels on Mediabunny's 0–1 scale (`Quality`). */
const QUALITY_SCALE: Record<ExportQuality, number> = {
  low: 0.25,
  medium: 0.5,
  high: 0.75,
  veryHigh: 1,
};

/** Bitrate of a 1080p H.264 picture at the reference quality (Mediabunny's estimate). */
const REFERENCE_BITRATE = 3_000_000;
const REFERENCE_PIXELS = 1920 * 1080;

/** Bitrates relative to H.264 at the same quality (Mediabunny's codec efficiency factors). */
const CODEC_EFFICIENCY: Record<'h264' | 'hevc' | 'av1' | 'vp9', number> = {
  h264: 1,
  hevc: 0.6,
  vp9: 0.6,
  av1: 0.4,
};

/**
 * Masters in bits per second at 1080p 29.97 fps (Apple's ProRes white paper: 422 HQ 220 Mb/s,
 * 4444 330 Mb/s; Avid DNxHR HQ about 180 Mb/s), scaled by pixels and frame rate.
 */
const MASTER_BITRATES_1080P30: Partial<Record<VideoExportCodec, number>> = {
  prores422Hq: 220_000_000,
  prores4444: 330_000_000,
  dnxhrHq: 180_000_000,
};

/** Container overhead (indexes, headers), as a fraction of the streams. */
const CONTAINER_OVERHEAD = 0.01;
/** 16-bit stereo PCM at 48 kHz. */
const PCM_BITRATE = 48_000 * 2 * 16;

/** Estimated bits per second of a picture (Mediabunny's formula when the encoder picks the rate). */
export function videoBitrate(video: VideoExportSettings, framesPerSecond: number): number {
  const pixels = video.width * video.height;
  const master = MASTER_BITRATES_1080P30[video.codec];
  if (master !== undefined) return master * (pixels / REFERENCE_PIXELS) * (framesPerSecond / 29.97);
  if (video.rate.kind === 'bitrate') return video.rate.bitsPerSecond;
  const efficiency = CODEC_EFFICIENCY[video.codec as keyof typeof CODEC_EFFICIENCY];
  const factor = 0.3 * Math.exp(2.5538 * QUALITY_SCALE[video.rate.level]);
  return REFERENCE_BITRATE * Math.pow(pixels / REFERENCE_PIXELS, 0.95) * efficiency * factor;
}

/**
 * Estimated size in bytes of an export of `seconds` (`≈` in the dialog): encoders choosing their
 * own rate (quality levels) may land far from it.
 */
export function estimatedBytes(
  settings: ExportSettings,
  seconds: number,
  framesPerSecond: number,
): number {
  const video = settings.video === null ? 0 : videoBitrate(settings.video, framesPerSecond);
  const audio =
    settings.audio === null
      ? 0
      : settings.audio.codec === 'pcm'
        ? PCM_BITRATE
        : settings.audio.bitsPerSecond;
  return Math.round(((video + audio) * seconds * (1 + CONTAINER_OVERHEAD)) / 8);
}
