import type {
  AudioEncoderChoice,
  AudioExportCodec,
  ExportContainer,
  ExportSettings,
  Format,
  VideoExportCodec,
  VideoRate,
} from '@se/protocol';

/** The shorter side of an export's picture: pixels, or the sequence's own size. */
export type ShortSide = number | 'sequence';

/** The picture of an export being set up (the output keeps the sequence's aspect). */
export interface DraftVideo {
  readonly codec: VideoExportCodec;
  readonly shortSide: ShortSide;
  readonly rate: VideoRate;
}

/** The sound of an export being set up (who encodes it is decided when it starts). */
export interface DraftAudio {
  readonly codec: AudioExportCodec;
  readonly bitsPerSecond: number;
}

/** What the export dialog edits: a container with an optional picture and sound. */
export interface ExportDraft {
  readonly container: ExportContainer;
  readonly video: DraftVideo | null;
  readonly audio: DraftAudio | null;
}

/** Picture codecs each container holds (the core checks the same table: `se-app` exports). */
export const CONTAINER_VIDEO_CODECS: Record<ExportContainer, readonly VideoExportCodec[]> = {
  mp4: ['h264', 'hevc', 'av1'],
  mov: ['h264', 'hevc', 'prores422Hq', 'prores4444', 'dnxhrHq'],
  webm: ['vp9', 'av1'],
  m4a: [],
  wav: [],
};

/** Shorter sides offered for a picture. */
export const SHORT_SIDES: readonly ShortSide[] = [720, 1080, 1440, 2160, 'sequence'];

/** Sound bitrates offered for AAC and Opus. */
export const AUDIO_BITRATES: readonly number[] = [128_000, 192_000, 256_000, 320_000];

/** Masters ffmpeg encodes from frames (their sound is PCM). */
export function isMasterCodec(codec: VideoExportCodec): boolean {
  return codec === 'prores422Hq' || codec === 'prores4444' || codec === 'dnxhrHq';
}

/** Sound codecs a container holds next to `video` (masters carry PCM sound). */
export function audioCodecsFor(
  container: ExportContainer,
  video: DraftVideo | null,
): readonly AudioExportCodec[] {
  if (video !== null && isMasterCodec(video.codec)) return ['pcm'];
  switch (container) {
    case 'mp4':
      return ['aac', 'opus'];
    case 'mov':
    case 'm4a':
      return ['aac'];
    case 'webm':
      return ['opus'];
    case 'wav':
      return ['pcm'];
  }
}

/** Whether a container holds sound only. */
export function isSoundOnly(container: ExportContainer): boolean {
  return CONTAINER_VIDEO_CODECS[container].length === 0;
}

const DEFAULT_VIDEO: DraftVideo = {
  codec: 'h264',
  shortSide: 1080,
  rate: { kind: 'quality', level: 'high' },
};
const DEFAULT_AUDIO_BITRATE = 192_000;

/**
 * `draft` made consistent after a change: a codec the container refuses becomes its first one,
 * sound-only containers lose the picture, video containers get one, and the sound follows the
 * codecs allowed next to the picture (kept off when it was off).
 */
export function normalizedDraft(draft: ExportDraft): ExportDraft {
  const videoCodecs = CONTAINER_VIDEO_CODECS[draft.container];
  const firstCodec = videoCodecs[0];
  let video: DraftVideo | null = null;
  if (firstCodec !== undefined) {
    const current = draft.video ?? { ...DEFAULT_VIDEO, codec: firstCodec };
    video = videoCodecs.includes(current.codec) ? current : { ...current, codec: firstCodec };
  }
  const audioCodecs = audioCodecsFor(draft.container, video);
  const firstAudio = audioCodecs[0] ?? 'aac';
  let audio = draft.audio;
  if (audio !== null && !audioCodecs.includes(audio.codec)) {
    audio = { codec: firstAudio, bitsPerSecond: audio.bitsPerSecond || DEFAULT_AUDIO_BITRATE };
  }
  if (audio === null && video === null) {
    audio = { codec: firstAudio, bitsPerSecond: DEFAULT_AUDIO_BITRATE };
  }
  return { container: draft.container, video, audio };
}

/** The even number nearest to `value` (at least 2). */
function even(value: number): number {
  return Math.max(2, Math.round(value / 2) * 2);
}

/**
 * The picture size of an export: the sequence's aspect with `shortSide` as its shorter side
 * (`sequence`: the sequence's own size), even sides, the longer side at most 7680 px.
 */
export function outputSize(
  format: Format,
  shortSide: ShortSide,
): { width: number; height: number } {
  if (shortSide === 'sequence') return { width: even(format.width), height: even(format.height) };
  const portrait = format.height > format.width;
  const short = Math.min(format.width, format.height);
  const long = Math.max(format.width, format.height);
  const scale = Math.min(shortSide / short, 7680 / long);
  const shortOut = even(short * scale);
  const longOut = even(long * scale);
  return portrait ? { width: shortOut, height: longOut } : { width: longOut, height: shortOut };
}

/** The settings the core receives for `draft` on a sequence of `format`. */
export function settingsOf(
  draft: ExportDraft,
  format: Format,
  audioEncoder: AudioEncoderChoice,
): ExportSettings {
  const video =
    draft.video === null
      ? null
      : {
          codec: draft.video.codec,
          ...outputSize(format, draft.video.shortSide),
          rate: draft.video.rate,
        };
  const audio =
    draft.audio === null
      ? null
      : {
          codec: draft.audio.codec,
          bitsPerSecond: draft.audio.codec === 'pcm' ? 0 : draft.audio.bitsPerSecond,
          encoder: draft.audio.codec === 'aac' ? audioEncoder : ('webCodecs' as const),
        };
  return { container: draft.container, video, audio };
}
