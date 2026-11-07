import type { Flicks, SequenceEntity } from '@se/protocol';
import type { MediaLocator } from '@se/engine/preview';
import { useEffect, useState, useSyncExternalStore } from 'react';

import type { TimeSpan } from '../../timeline';
import type { AudioEngineFactory } from '../services/audio-engine-factory';
import { PlaybackSession } from '../services/playback-session';
import type { ViewerEngineController } from '../services/viewer-engine-controller';
import type { PlaybackScene } from '../services/viewer-scene';

/** What playback plays and how. */
export interface PlaybackSessionInputs {
  readonly createAudio: AudioEngineFactory;
  readonly engine: ViewerEngineController;
  readonly sequence: SequenceEntity;
  readonly scene: PlaybackScene | null;
  readonly contentEnd: Flicks;
  readonly locator: MediaLocator;
  readonly loopRange: TimeSpan | null;
  readonly looping: boolean;
  readonly scrubAudio: boolean;
}

/**
 * The viewer's playback session (outside React): started on mount, stopped on unmount (React
 * StrictMode does both twice), fed with the inputs as they change.
 */
export function usePlaybackSession(inputs: PlaybackSessionInputs): PlaybackSession {
  const { createAudio, engine, sequence, scene, contentEnd, locator } = inputs;
  const { loopRange, looping, scrubAudio } = inputs;
  const [session] = useState(() => new PlaybackSession({ createAudio, picture: engine }));

  useEffect(() => {
    session.start();
    return () => {
      session.stop();
    };
  }, [session]);
  useEffect(() => {
    session.setTarget({ sequenceId: sequence.id, frameRate: sequence.format.frameRate });
  }, [session, sequence.id, sequence.format.frameRate]);
  useEffect(() => {
    session.setLocator(locator);
  }, [session, locator]);
  useEffect(() => {
    session.setScene(scene, contentEnd);
  }, [session, scene, contentEnd]);
  useEffect(() => {
    session.setLoopRange(loopRange);
  }, [session, loopRange]);
  useEffect(() => {
    session.setLooping(looping);
  }, [session, looping]);
  useEffect(() => {
    session.setScrubAudio(scrubAudio);
  }, [session, scrubAudio]);

  return session;
}

/** The playback rate and loop state, re-rendering when they change (not on every frame). */
export function usePlaybackState(session: PlaybackSession) {
  return useSyncExternalStore(session.subscribe, session.getState);
}
