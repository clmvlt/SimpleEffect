import { gainToDb } from '../mix';
import { METER_PROCESSOR_NAME, type LevelReport } from './level-math';

/** Levels of the master output in dBFS (`-Infinity` for silence), left and right. */
export interface MeterLevels {
  readonly peakDb: readonly number[];
  readonly rmsDb: readonly number[];
}

/** Silence on both channels. */
export const SILENT_LEVELS: MeterLevels = {
  peakDb: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
  rmsDb: [Number.NEGATIVE_INFINITY, Number.NEGATIVE_INFINITY],
};

function isReport(value: unknown): value is LevelReport {
  if (typeof value !== 'object' || value === null) return false;
  const report = value as Partial<LevelReport>;
  return Array.isArray(report.peak) && Array.isArray(report.rms);
}

/**
 * The master level meter: an AudioWorklet measuring peak and RMS per channel on the audio thread
 * (nothing per sample on the main thread); `levels()` returns the latest report. Its output is
 * silent, connected only so the audio thread keeps it running.
 */
export class AudioMeter {
  private latest: MeterLevels = SILENT_LEVELS;

  private constructor(readonly node: AudioWorkletNode) {
    node.port.onmessage = (event: MessageEvent<unknown>) => {
      if (!isReport(event.data)) return;
      this.latest = {
        peakDb: event.data.peak.map((level) => gainToDb(level)),
        rmsDb: event.data.rms.map((level) => gainToDb(level)),
      };
    };
  }

  /** Loads the processor module at `moduleUrl` and creates the meter on `context`. */
  static async create(context: AudioContext, moduleUrl: string): Promise<AudioMeter> {
    await context.audioWorklet.addModule(moduleUrl);
    const node = new AudioWorkletNode(context, METER_PROCESSOR_NAME, {
      numberOfInputs: 1,
      numberOfOutputs: 1,
      outputChannelCount: [2],
      channelCount: 2,
      channelCountMode: 'explicit',
      channelInterpretation: 'speakers',
    });
    node.connect(context.destination);
    return new AudioMeter(node);
  }

  levels(): MeterLevels {
    return this.latest;
  }

  dispose(): void {
    this.node.port.onmessage = null;
    this.node.disconnect();
  }
}
