import { PEAK_LEVELS, type PeakLevel, type WaveformPeaks } from '@se/protocol';

/**
 * The waveform level to fetch at a zoom: the coarsest level that still gives at least one peak
 * per pixel (fewer bytes, no visible loss), the finest one when zoomed in further.
 */
export function peakLevelFor(
  sampleRate: number,
  speed: number,
  pixelsPerSecond: number,
): PeakLevel {
  const samplesPerPixel = (sampleRate * speed) / pixelsPerSecond;
  const fitting = PEAK_LEVELS.filter((level) => level <= samplesPerPixel);
  return fitting.at(-1) ?? PEAK_LEVELS[0];
}

/** Peak values are 16-bit samples: this maps them to −1…1. */
const SAMPLE_SCALE = 32_768;

/** What part of a clip a waveform canvas covers. */
export interface WaveformWindow {
  /** Source time (seconds) at the clip's start. */
  readonly sourceStartSeconds: number;
  /** Playback speed of the clip (source seconds per timeline second). */
  readonly speed: number;
  /** Zoom of the timeline. */
  readonly pixelsPerSecond: number;
  /** Pixels of the clip hidden left of the canvas (the canvas only covers what is visible). */
  readonly offsetPixels: number;
  /** Width of the canvas in pixels (one column per pixel). */
  readonly width: number;
}

/** Minimum and maximum sample (−1…1) of each pixel column, all channels together. */
export interface WaveformColumns {
  readonly minimums: Float32Array;
  readonly maximums: Float32Array;
}

/**
 * The envelope to draw: for each pixel column, the extremes of the peak blocks it covers. Columns
 * past the end of the media stay at zero.
 */
export function waveformColumns(peaks: WaveformPeaks, window: WaveformWindow): WaveformColumns {
  const width = Math.max(0, Math.floor(window.width));
  const minimums = new Float32Array(width);
  const maximums = new Float32Array(width);
  const blocksPerSecond = peaks.sampleRate / peaks.samplesPerPeak;
  const sourceSecondsPerPixel = window.speed / window.pixelsPerSecond;
  const stride = peaks.channels * 2;
  for (let column = 0; column < width; column += 1) {
    const from = window.sourceStartSeconds + (window.offsetPixels + column) * sourceSecondsPerPixel;
    const first = Math.floor(from * blocksPerSecond);
    const last = Math.max(first + 1, Math.floor((from + sourceSecondsPerPixel) * blocksPerSecond));
    let low = 0;
    let high = 0;
    for (let block = Math.max(0, first); block < Math.min(last, peaks.peakCount); block += 1) {
      for (let channel = 0; channel < peaks.channels; channel += 1) {
        const offset = block * stride + channel * 2;
        low = Math.min(low, peaks.values[offset] ?? 0);
        high = Math.max(high, peaks.values[offset + 1] ?? 0);
      }
    }
    minimums[column] = low / SAMPLE_SCALE;
    maximums[column] = high / SAMPLE_SCALE;
  }
  return { minimums, maximums };
}
