/**
 * Waveform peaks as served by `/media/assets/{id}/peaks/{level}` (the `SEPK` format documented in
 * `crates/se-media/CLAUDE.md`): per block of `samplesPerPeak` samples, per channel, the minimum and
 * maximum sample value as `Int16` (±32767 = full scale).
 */
export interface WaveformPeaks {
  /** Channels per block (1 or 2). */
  readonly channels: number;
  /** Samples per second of the source. */
  readonly sampleRate: number;
  /** Samples (per channel) summarised by one block. */
  readonly samplesPerPeak: number;
  /** Number of blocks. */
  readonly peakCount: number;
  /**
   * `peakCount × channels × 2` values, block-major: block 0 channel 0 min, max, block 0 channel 1
   * min, max…
   */
  readonly values: Int16Array;
}

const MAGIC = 'SEPK';
const FORMAT_VERSION = 1;
const HEADER_BYTES = 32;

/**
 * Reads a `SEPK` file. Throws an `Error` with the reason when the data is not a valid file of
 * the supported version.
 */
export function decodeWaveformPeaks(buffer: ArrayBuffer): WaveformPeaks {
  if (buffer.byteLength < HEADER_BYTES) {
    throw new Error('the file is shorter than the SEPK header');
  }
  const view = new DataView(buffer);
  const magic = String.fromCharCode(
    view.getUint8(0),
    view.getUint8(1),
    view.getUint8(2),
    view.getUint8(3),
  );
  if (magic !== MAGIC) {
    throw new Error('not a SEPK file');
  }
  const version = view.getUint16(4, true);
  if (version !== FORMAT_VERSION) {
    throw new Error(`unknown SEPK version ${String(version)}`);
  }
  const channels = view.getUint16(6, true);
  const peakCount = view.getUint32(16, true);
  const valueCount = peakCount * channels * 2;
  if (buffer.byteLength !== HEADER_BYTES + valueCount * 2) {
    throw new Error('the SEPK body does not match its peak count');
  }
  // Every supported platform is little-endian, so the body maps directly onto an Int16Array.
  return {
    channels,
    sampleRate: view.getUint32(8, true),
    samplesPerPeak: view.getUint32(12, true),
    peakCount,
    values: new Int16Array(buffer, HEADER_BYTES, valueCount),
  };
}
