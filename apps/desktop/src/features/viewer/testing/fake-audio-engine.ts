import type { Flicks } from '@se/protocol';
import type { AudioEngine, AudioEngineStats, AudioScene, MeterLevels } from '@se/engine/audio';
import type { PlaybackBounds, PlaybackClock, Transport } from '@se/engine/playback';

import type { AudioEngineFactory } from '../services/audio-engine-factory';

/** An audio engine recording what playback asks of it, on a clock the test moves. */
export class FakeAudioEngine implements AudioEngine {
  now = 0;
  readonly clock: PlaybackClock = { now: () => this.now };
  scene: AudioScene | null = null;
  transport: Transport | null = null;
  bounds: PlaybackBounds | null = null;
  readonly scrubs: Flicks[] = [];
  resumed = 0;
  destroyed = false;
  meter: MeterLevels = { peakDb: [-6, -12], rmsDb: [-12, -18] };

  setScene(scene: AudioScene | null): void {
    this.scene = scene;
  }
  setLocator(): void {}
  setTransport(transport: Transport | null, bounds: PlaybackBounds): void {
    this.transport = transport;
    this.bounds = bounds;
  }
  scrub(time: Flicks): void {
    this.scrubs.push(time);
  }
  resume(): Promise<void> {
    this.resumed += 1;
    return Promise.resolve();
  }
  levels(): MeterLevels {
    return this.meter;
  }
  stats(): AudioEngineStats {
    return { chunks: 4, bytes: 1_536_000, loading: 0, voices: 2 };
  }
  destroy(): void {
    this.destroyed = true;
  }
}

/** A factory handing out {@link FakeAudioEngine}s, keeping every engine it made. */
export function fakeAudioFactory(): {
  readonly factory: AudioEngineFactory;
  readonly engines: FakeAudioEngine[];
} {
  const engines: FakeAudioEngine[] = [];
  return {
    engines,
    factory: () => {
      const engine = new FakeAudioEngine();
      engines.push(engine);
      return Promise.resolve(engine);
    },
  };
}
