import type { WaveformPeaks } from '@se/protocol';

/** Full scale of the stored peak values. */
const FULL_SCALE = 32767;

/**
 * An SVG path drawing the waveform in a `width` × `height` box: one vertical bar per column, from
 * the lowest to the highest sample of the blocks under it, all channels merged.
 */
export function waveformPath(peaks: WaveformPeaks, width: number, height: number): string {
  const columns = Math.max(1, Math.floor(width));
  if (peaks.peakCount === 0) {
    return '';
  }
  const middle = height / 2;
  const segments: string[] = [];
  for (let column = 0; column < columns; column += 1) {
    const first = Math.floor((column * peaks.peakCount) / columns);
    const last = Math.max(first + 1, Math.floor(((column + 1) * peaks.peakCount) / columns));
    let low = 0;
    let high = 0;
    for (let block = first; block < last && block < peaks.peakCount; block += 1) {
      for (let channel = 0; channel < peaks.channels; channel += 1) {
        const offset = (block * peaks.channels + channel) * 2;
        low = Math.min(low, peaks.values[offset] ?? 0);
        high = Math.max(high, peaks.values[offset + 1] ?? 0);
      }
    }
    const top = middle - (high / FULL_SCALE) * middle;
    // At least half a pixel, so silence still shows as a line.
    const bottom = Math.max(middle - (low / FULL_SCALE) * middle, top + 0.5);
    segments.push(`M${String(column + 0.5)} ${top.toFixed(1)}V${bottom.toFixed(1)}`);
  }
  return segments.join('');
}
