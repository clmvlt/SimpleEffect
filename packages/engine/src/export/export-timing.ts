import type { Flicks, FrameRate } from '@se/protocol';

import { frameDurationFlicks } from '../playback/frame-pacing';
import { FLICKS_PER_SECOND } from '../time/flicks';

/** Sample rate of exported sound (every codec of the export dialog takes it). */
export const EXPORT_SAMPLE_RATE = 48_000;
/** Exported sound is mixed offline in blocks of this many seconds (ADR-0038). */
export const AUDIO_BLOCK_SECONDS = 30;
/** Channels of exported sound (stereo: mono sources are spread to both, like playback). */
export const EXPORT_CHANNELS = 2;

/** The frames of an exported span `[start, end)` at the sequence's frame rate. */
export interface ExportFrames {
  /** Frames written: those starting before `end`. */
  readonly count: number;
  /** Seconds of one frame. */
  readonly frameSeconds: number;
  /** Sequence time drawn for frame `index`. */
  timeOf(index: number): Flicks;
  /** File time of frame `index`, in seconds (frame 0 at 0). */
  secondsOf(index: number): number;
}

/** The frames of `[start, end)` at `rate` (pure; frame `k` is drawn at `start + k` frames). */
export function exportFrames(start: Flicks, end: Flicks, rate: FrameRate): ExportFrames {
  const duration = frameDurationFlicks(rate);
  const count = Math.max(0, Math.ceil((end - start) / duration - 1e-9));
  const frameSeconds = rate.denominator / rate.numerator;
  return {
    count,
    frameSeconds,
    timeOf: (index) => (start + Math.round(index * duration)) as Flicks,
    secondsOf: (index) => (index * rate.denominator) / rate.numerator,
  };
}

/** One block of exported sound. */
export interface AudioBlock {
  readonly index: number;
  /** Sequence time of its first sample. */
  readonly start: Flicks;
  /** File time of its first sample, in seconds. */
  readonly seconds: number;
  /** Samples per channel. */
  readonly frames: number;
}

/** Samples per channel of `[start, end)` at `sampleRate`. */
export function sampleCount(start: Flicks, end: Flicks, sampleRate: number): number {
  return Math.max(0, Math.round(((end - start) * sampleRate) / FLICKS_PER_SECOND));
}

/**
 * The blocks of sound of `[start, end)` (pure): `blockSeconds` each, the last one shorter. Block
 * boundaries fall on whole samples and whole flicks, so consecutive blocks join on the sample
 * and their lengths add up to the exact duration.
 */
export function audioBlocks(
  start: Flicks,
  end: Flicks,
  sampleRate: number = EXPORT_SAMPLE_RATE,
  blockSeconds: number = AUDIO_BLOCK_SECONDS,
): AudioBlock[] {
  const total = sampleCount(start, end, sampleRate);
  const perBlock = Math.round(blockSeconds * sampleRate);
  const blocks: AudioBlock[] = [];
  for (let index = 0, first = 0; first < total; index++, first += perBlock) {
    blocks.push({
      index,
      start: (start + index * blockSeconds * FLICKS_PER_SECOND) as Flicks,
      seconds: index * blockSeconds,
      frames: Math.min(perBlock, total - first),
    });
  }
  return blocks;
}
