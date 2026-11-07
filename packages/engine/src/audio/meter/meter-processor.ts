/**
 * AudioWorklet processor of the level meter (`AudioMeter`): measures the peak and RMS of each
 * channel on the audio thread and posts them about 30 times per second. Loaded as its own module
 * (`?worker&url`), so it only imports pure code.
 */
import { LevelAccumulator, METER_PROCESSOR_NAME } from './level-math';

/** Reports per second. */
const REPORT_RATE = 30;

/** The globals of an `AudioWorkletGlobalScope` this module uses (not in the DOM typings). */
interface WorkletScope {
  readonly sampleRate: number;
  readonly AudioWorkletProcessor: new () => { readonly port: MessagePort };
  registerProcessor(name: string, processor: new () => object): void;
}

const scope = globalThis as unknown as Partial<WorkletScope>;

if (scope.AudioWorkletProcessor !== undefined && scope.registerProcessor !== undefined) {
  const sampleRate = scope.sampleRate ?? 48_000;

  class MeterProcessor extends scope.AudioWorkletProcessor {
    private readonly levels = new LevelAccumulator(2, Math.round(sampleRate / REPORT_RATE));

    process(inputs: readonly (readonly Float32Array[])[]): boolean {
      const report = this.levels.add(inputs[0] ?? []);
      if (report !== null) this.port.postMessage(report);
      return true;
    }
  }

  scope.registerProcessor(METER_PROCESSOR_NAME, MeterProcessor);
}
