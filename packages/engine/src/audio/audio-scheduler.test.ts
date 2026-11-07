import type { AssetId, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { clip, item, seconds, still } from '../testing/document';
import type { PlaybackBounds, Transport } from '../playback/transport';
import type { AudioChunk } from './audio-chunk';
import { audioClipsOf } from './audio-clips';
import {
  AudioScheduler,
  SCHEDULE_AHEAD_SECONDS,
  type AudioChunkProvider,
  type AudioOutput,
  type AudioVoiceRequest,
} from './audio-scheduler';

const ASSET = 'asset-sound' as AssetId;
const BOUNDS: PlaybackBounds = { span: { start: 0 as Flicks, end: seconds(30) }, loop: false };

/** A clock and a recording output. */
class FakeOutput implements AudioOutput {
  now = 10;
  readonly requests: (AudioVoiceRequest & { stopped: boolean })[] = [];
  currentTime(): number {
    return this.now;
  }
  play(request: AudioVoiceRequest) {
    const recorded = { ...request, stopped: false };
    this.requests.push(recorded);
    return { stop: () => (recorded.stopped = true) };
  }
  live() {
    return this.requests.filter((request) => !request.stopped);
  }
}

/** Chunks decoded instantly, or on demand when `manual`. */
class FakeChunks implements AudioChunkProvider {
  readonly cached = new Map<number, AudioChunk>();
  readonly loads: number[] = [];
  manual = false;
  private readonly waiting: (() => void)[] = [];
  peek(_assetId: AssetId, index: number) {
    return this.cached.get(index) ?? null;
  }
  load(_assetId: AssetId, index: number): Promise<AudioChunk | null> {
    this.loads.push(index);
    const chunk = { buffer: { duration: 1 } as AudioBuffer, start: index };
    if (!this.manual) {
      this.cached.set(index, chunk);
      return Promise.resolve(chunk);
    }
    return new Promise((resolve) =>
      this.waiting.push(() => {
        this.cached.set(index, chunk);
        resolve(chunk);
      }),
    );
  }
  release(): Promise<void> {
    for (const resolve of this.waiting.splice(0)) resolve();
    return Promise.resolve();
  }
}

function sceneClips(
  options: Parameters<typeof clip>[1] = {},
  start = 0 as Flicks,
  length = seconds(5),
) {
  const music = item(start, length, clip(ASSET, options));
  return audioClipsOf({
    tracks: [
      {
        id: 'a1',
        kind: 'audio',
        muted: false,
        solo: false,
        volumeDb: 0,
        pan: 0,
        hidden: false,
        items: [music],
      },
    ],
  });
}

function setup(clips = sceneClips()) {
  const output = new FakeOutput();
  const chunks = new FakeChunks();
  for (let index = 0; index < 10; index++) void chunks.load(ASSET, index);
  chunks.loads.length = 0;
  const scheduler = new AudioScheduler(output, chunks);
  scheduler.setClips(clips);
  /** Ticks every 25 ms for `duration` seconds. */
  const run = (duration: number) => {
    const end = output.now + duration;
    while (output.now < end) {
      output.now += 0.025;
      scheduler.tick();
    }
  };
  return { output, chunks, scheduler, run };
}

const playFrom = (time: Flicks, anchorClock: number): Transport => ({
  rate: 1,
  anchorTime: time,
  anchorClock,
});

describe('AudioScheduler', () => {
  it('places chunk pieces end to end, exactly on the anchor', () => {
    const { output, scheduler, run } = setup();
    scheduler.setTransport(playFrom(0 as Flicks, 10.05), BOUNDS);
    run(3);
    const voices = output.live();
    expect(voices.map((voice) => voice.when - 10.05)).toEqual([0, 1, 2, 3]);
    for (let index = 1; index < voices.length; index++) {
      const previous = voices[index - 1];
      const voice = voices[index];
      if (previous === undefined || voice === undefined) continue;
      expect(previous.when + previous.duration).toBeCloseTo(voice.when, 12);
      expect(voice.offset).toBe(0);
    }
  });

  it('schedules only within the anticipation window', () => {
    const { output, scheduler, run } = setup();
    scheduler.setTransport(playFrom(0 as Flicks, 10), BOUNDS);
    run(0.7);
    expect(output.live()).toHaveLength(1);
    run(0.2);
    expect(output.live()).toHaveLength(2);
    expect(output.live()[1]?.when).toBeLessThanOrEqual(output.now + SCHEDULE_AHEAD_SECONDS);
  });

  it('restarts mid-chunk on a seek and stops what was scheduled', () => {
    const { output, scheduler } = setup();
    scheduler.setTransport(playFrom(0 as Flicks, 10), BOUNDS);
    const before = output.requests.length;
    scheduler.setTransport(playFrom(seconds(2.25), 10.05), BOUNDS);
    expect(output.requests.slice(0, before).every((request) => request.stopped)).toBe(true);
    const [first] = output.live();
    expect(first?.when).toBe(10.05);
    expect(first?.offset).toBeCloseTo(0.25, 9);
    expect(first?.duration).toBeCloseTo(0.75, 9);
  });

  it('maps source time with speed and source start', () => {
    const clips = sceneClips({ sourceStart: seconds(0.5), speed: 2 }, seconds(1));
    const { output, scheduler, run } = setup(clips);
    scheduler.setTransport(playFrom(0 as Flicks, 10), BOUNDS);
    run(1.5);
    const [first, second] = output.live();
    // The clip starts 1 s in and plays source 0.5 s → chunk 0 from 0.5, then chunk 1 at 1.25 s.
    expect(first).toMatchObject({ when: 11, offset: 0.5, playbackRate: 2 });
    expect(first?.duration).toBeCloseTo(0.5, 9);
    expect(second?.when).toBeCloseTo(11.25, 9);
  });

  it('crosses a loop wrap without a gap', () => {
    const { output, scheduler, run } = setup();
    const loop: PlaybackBounds = { span: { start: seconds(1), end: seconds(1.5) }, loop: true };
    scheduler.setTransport(playFrom(seconds(1), 10.05), loop);
    run(0.6);
    const [first, second] = output.live();
    expect(first).toMatchObject({ when: 10.05, offset: 0 });
    expect(first?.duration).toBeCloseTo(0.5, 9);
    expect(second?.when).toBeCloseTo(10.55, 9);
    expect(second?.offset).toBe(0);
  });

  it('trims sound decoded late so it joins playback in time', async () => {
    const { output, scheduler, chunks } = setup();
    chunks.cached.clear();
    chunks.manual = true;
    scheduler.setTransport(playFrom(0 as Flicks, 10), BOUNDS);
    output.now = 10.1;
    await chunks.release();
    await Promise.resolve();
    const [first] = output.live();
    expect(first?.when).toBeCloseTo(10.105, 9);
    expect(first?.offset).toBeCloseTo(0.105, 9);
  });

  it('is silent at other speeds and plays fading scrub snippets when paused', () => {
    const { output, scheduler, run } = setup();
    scheduler.setTransport({ rate: 2, anchorTime: 0, anchorClock: 10 }, BOUNDS);
    run(1);
    expect(output.requests).toHaveLength(0);
    scheduler.setTransport(null, BOUNDS);
    scheduler.scrub(seconds(1.5));
    scheduler.scrub(seconds(1.6));
    expect(output.requests).toHaveLength(1);
    expect(output.requests[0]).toMatchObject({ offset: 0.5, fade: 0.005 });
    expect(scheduler.busy).toBe(true);
    run(0.05);
    expect(output.requests).toHaveLength(2);
    expect(output.requests[1]?.offset).toBeCloseTo(0.6, 9);
  });

  it('decodes ahead and keeps voices when an edit leaves the sound unchanged', () => {
    const { scheduler, chunks, output, run } = setup();
    chunks.cached.clear();
    scheduler.setTransport(playFrom(0 as Flicks, 10), BOUNDS);
    run(0.05);
    expect(new Set(chunks.loads)).toEqual(new Set([0, 1, 2]));
    const voices = output.live().length;
    scheduler.setClips(sceneClips());
    expect(output.live()).toHaveLength(voices);
    scheduler.setClips(sceneClips({ volumeDb: still(-6) } as never));
    expect(output.live()).toHaveLength(voices);
  });
});
