/** Name the meter processor is registered under in the AudioWorklet scope. */
export const METER_PROCESSOR_NAME = 'se-level-meter';

/** Levels of one report, linear (0…1), per channel. */
export interface LevelReport {
  readonly peak: readonly number[];
  readonly rms: readonly number[];
}

/**
 * Accumulates audio blocks (128 frames each in an AudioWorklet) and yields the peak and RMS of
 * each channel every `framesPerReport` frames. Allocation-free per block; a mono block counts
 * for every channel, a block without input counts as silence.
 */
export class LevelAccumulator {
  private readonly peak: Float64Array;
  private readonly squares: Float64Array;
  private frames = 0;

  constructor(
    private readonly channels: number,
    private readonly framesPerReport: number,
  ) {
    this.peak = new Float64Array(channels);
    this.squares = new Float64Array(channels);
  }

  /** Adds one block (`frames` long when it has no channel); returns a report when one is due. */
  add(block: readonly Float32Array[], frames = 128): LevelReport | null {
    const length = block[0]?.length ?? frames;
    for (let channel = 0; channel < this.channels; channel++) {
      const samples = block[Math.min(channel, block.length - 1)];
      if (samples === undefined) continue;
      let peak = this.peak[channel] ?? 0;
      let squares = this.squares[channel] ?? 0;
      for (let index = 0; index < samples.length; index++) {
        const sample = samples[index] ?? 0;
        const level = Math.abs(sample);
        if (level > peak) peak = level;
        squares += sample * sample;
      }
      this.peak[channel] = peak;
      this.squares[channel] = squares;
    }
    this.frames += length;
    return this.frames >= this.framesPerReport ? this.report() : null;
  }

  private report(): LevelReport {
    const peak = Array.from(this.peak);
    const rms = Array.from(this.squares, (squares) => Math.sqrt(squares / this.frames));
    this.peak.fill(0);
    this.squares.fill(0);
    this.frames = 0;
    return { peak, rms };
  }
}
