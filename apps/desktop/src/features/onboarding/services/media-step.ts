import type { CodecSupport, MediaToolsStatus, VisionStatus } from '@se/protocol';

import type { CheckTone } from './ai-step';

/** A row of the « Binaires & GPU » step: its tone and the i18n key of its text. */
export interface MediaCheck {
  readonly tone: CheckTone;
  /** Key under `onboarding.media.`. */
  readonly key:
    | 'tools.checking'
    | 'tools.missing'
    | 'tools.softwareOnly'
    | 'tools.ready'
    | 'vision.checking'
    | 'vision.unavailable'
    | 'vision.ready'
    | 'gpu.checking'
    | 'gpu.unknown'
    | 'gpu.ready'
    | 'codecs.checking'
    | 'codecs.summary';
  /** Values interpolated into the text. */
  readonly values: MediaTextValues;
}

/** Values interpolated into the texts of the « Binaires & GPU » rows. */
export interface MediaTextValues {
  readonly version?: string;
  readonly encoders?: string;
  readonly problem?: string;
  readonly reason?: string;
  readonly runtime?: string;
  readonly accelerator?: string;
  readonly name?: string;
  readonly decode?: string;
  readonly encode?: string;
}

/** The ffmpeg row: found with its version and hardware encoders, or why not. */
export function toolsCheck(tools: MediaToolsStatus | 'failed' | null): MediaCheck {
  if (tools === null) return { tone: 'neutral', key: 'tools.checking', values: {} };
  if (tools === 'failed' || tools.ffmpegPath === null || tools.ffprobePath === null) {
    const problem = tools === 'failed' ? '' : (tools.problem ?? '');
    return { tone: 'danger', key: 'tools.missing', values: { problem } };
  }
  const encoders = tools.hardwareEncoders.join(', ');
  return {
    tone: 'success',
    key: encoders === '' ? 'tools.softwareOnly' : 'tools.ready',
    values: { version: tools.ffmpegVersion ?? '?', encoders },
  };
}

/** The face detection row (automatic reframing). */
export function visionCheck(vision: VisionStatus | null): MediaCheck {
  if (vision === null) return { tone: 'neutral', key: 'vision.checking', values: {} };
  if (!vision.available) {
    return { tone: 'warning', key: 'vision.unavailable', values: { reason: vision.reason ?? '' } };
  }
  return {
    tone: 'success',
    key: 'vision.ready',
    values: { runtime: vision.runtime ?? 'ONNX Runtime', accelerator: vision.accelerator ?? 'CPU' },
  };
}

/** The GPU row (`undefined` while checking, `null` when WebGL hides it). */
export function gpuCheck(gpu: string | null | undefined): MediaCheck {
  if (gpu === undefined) return { tone: 'neutral', key: 'gpu.checking', values: {} };
  if (gpu === null) return { tone: 'warning', key: 'gpu.unknown', values: {} };
  return { tone: 'success', key: 'gpu.ready', values: { name: gpu } };
}

/**
 * The codecs row: decoding H.264 matters for smooth previews (others fall back to mezzanines),
 * encoding it makes exports fast.
 */
export function codecsCheck(codecs: readonly CodecSupport[] | null): MediaCheck {
  if (codecs === null) return { tone: 'neutral', key: 'codecs.checking', values: {} };
  const list = (pick: (codec: CodecSupport) => boolean) => {
    const names = codecs.filter(pick).map((codec) => codec.codec);
    return names.length === 0 ? '—' : names.join(', ');
  };
  const h264 = codecs.find((codec) => codec.codec === 'H.264');
  const tone: CheckTone = h264?.decode !== true ? 'danger' : h264.encode ? 'success' : 'warning';
  return {
    tone,
    key: 'codecs.summary',
    values: { decode: list((codec) => codec.decode), encode: list((codec) => codec.encode) },
  };
}
