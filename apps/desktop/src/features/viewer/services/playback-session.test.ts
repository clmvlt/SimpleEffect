import type { Flicks } from '@se/protocol';
import type { FrameTicker } from '@se/engine/playback';
import type { PreviewPlayback } from '@se/engine/preview';
import { beforeEach, describe, expect, it } from 'vitest';

import { playbackStatusStore } from '../../../shared/playback';
import { IDS } from '../../../shared/project-store/testing/session-fixture';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../../timeline';
import { fakeAudioFactory, type FakeAudioEngine } from '../testing/fake-audio-engine';
import { AudioUnsupportedError, type AudioEngineFactory } from './audio-engine-factory';
import { PlaybackSession } from './playback-session';

const SECOND = 705_600_000 as Flicks;
const RATE = { numerator: 25, denominator: 1 };

function setup(createAudio: AudioEngineFactory = fakeAudioFactory().factory) {
  let pending: (() => void) | null = null;
  const ticker: FrameTicker = {
    request: (callback) => {
      pending = callback;
      return 1;
    },
    cancel: () => {
      pending = null;
    },
  };
  const pictures: (PreviewPlayback | null)[] = [];
  const session = new PlaybackSession({
    createAudio,
    ticker,
    picture: { setPlayback: (playback) => pictures.push(playback) },
    fallbackClock: { now: () => 0 },
  });
  session.setTarget({ sequenceId: IDS.sequence, frameRate: RATE });
  session.setScene(null, (10 * SECOND) as Flicks);
  session.start();
  return {
    session,
    pictures,
    refresh: () => {
      const callback = pending;
      pending = null;
      callback?.();
    },
    playhead: () => playheadTimeOf(playheadStore.getState(), IDS.sequence),
  };
}

async function audioOf(engines: FakeAudioEngine[]): Promise<FakeAudioEngine> {
  await Promise.resolve();
  const engine = engines.at(-1);
  if (engine === undefined) throw new Error('no audio engine');
  return engine;
}

beforeEach(() => {
  playheadStore.setState({ times: {} });
  playbackStatusStore.setState({ rate: 0 });
});

describe('PlaybackSession', () => {
  it('plays on the audio clock, moving the playhead and the picture engine', async () => {
    const audio = fakeAudioFactory();
    const { session, pictures, refresh, playhead } = setup(audio.factory);
    const engine = await audioOf(audio.engines);
    session.togglePlay();
    expect(engine.resumed).toBe(1);
    expect(engine.transport?.rate).toBe(1);
    expect(pictures.at(-1)).toEqual({ rate: 1, frameRate: RATE });
    expect(playbackStatusStore.getState().rate).toBe(1);
    engine.now = 1.53;
    refresh();
    expect(playhead()).toBe(1.52 * SECOND);
    session.pause();
    expect(engine.transport).toBeNull();
    expect(pictures.at(-1)).toBeNull();
  });

  it('treats a playhead moved elsewhere as a seek, with a scrub snippet when paused', async () => {
    const audio = fakeAudioFactory();
    const { session, refresh, playhead } = setup(audio.factory);
    const engine = await audioOf(audio.engines);
    setPlayheadTime(IDS.sequence, (2 * SECOND) as Flicks);
    expect(engine.scrubs).toEqual([2 * SECOND]);
    session.setScrubAudio(false);
    setPlayheadTime(IDS.sequence, (3 * SECOND) as Flicks);
    expect(engine.scrubs).toHaveLength(1);
    session.togglePlay();
    setPlayheadTime(IDS.sequence, (6 * SECOND) as Flicks);
    expect(engine.transport?.anchorTime).toBe(6 * SECOND);
    engine.now = 0.53;
    refresh();
    expect(playhead()).toBe(6.52 * SECOND);
  });

  it('steps frames with a snippet, stops back at the start, goes to the edges', async () => {
    const audio = fakeAudioFactory();
    const { session, refresh, playhead } = setup(audio.factory);
    const engine = await audioOf(audio.engines);
    session.step(5);
    expect(playhead()).toBe(0.2 * SECOND);
    expect(engine.scrubs).toEqual([0.2 * SECOND]);
    session.togglePlay();
    engine.now = 1;
    refresh();
    session.stopPlayback();
    expect(playhead()).toBe(0.2 * SECOND);
    session.setLoopRange({ start: SECOND, end: (2 * SECOND) as Flicks });
    session.setLooping(true);
    session.goToEnd();
    expect(playhead()).toBe(10 * SECOND);
    session.goToStart();
    expect(playhead()).toBe(0);
  });

  it('plays silent on the fallback clock without audio, and restarts after a stop', async () => {
    const failing: AudioEngineFactory = () => Promise.reject(new AudioUnsupportedError());
    const { session } = setup(failing);
    await Promise.resolve();
    expect(session.levels().peakDb).toEqual([-Infinity, -Infinity]);
    expect(session.audioStats()).toBeNull();
    const audio = fakeAudioFactory();
    const restarted = setup(audio.factory);
    const first = await audioOf(audio.engines);
    restarted.session.stop();
    expect(first.destroyed).toBe(true);
    restarted.session.start();
    const second = await audioOf(audio.engines);
    expect(second).not.toBe(first);
    expect(restarted.session.levels().peakDb).toEqual([-6, -12]);
  });
});
