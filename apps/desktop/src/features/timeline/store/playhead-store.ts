import type { Flicks, SequenceId } from '@se/protocol';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { ZERO_FLICKS } from '../../../shared/time';

/**
 * Playhead position per sequence. Kept apart from the view store because it moves often (drags,
 * playback from prompt 13): the playhead line subscribes to it and moves itself by style,
 * without re-rendering the timeline.
 */
export interface PlayheadState {
  readonly times: Readonly<Record<SequenceId, Flicks>>;
}

/** The playhead store. */
export const playheadStore = createStore<PlayheadState>()(() => ({ times: {} }));

/** The playhead time of `sequenceId` in `state` (zero before it ever moved). */
export function playheadTimeOf(state: PlayheadState, sequenceId: SequenceId): Flicks {
  return state.times[sequenceId] ?? ZERO_FLICKS;
}

/** Moves the playhead of `sequenceId` to `time` (callers snap it to a frame). */
export function setPlayheadTime(sequenceId: SequenceId, time: Flicks): void {
  const state = playheadStore.getState();
  if (state.times[sequenceId] === time) return;
  playheadStore.setState({ times: { ...state.times, [sequenceId]: time } });
}

/**
 * The playhead time of `sequenceId`, re-rendering on every move: for readouts only (the timecode
 * field); moving elements subscribe to the store instead.
 */
export function usePlayheadTime(sequenceId: SequenceId): Flicks {
  return useStore(playheadStore, (state) => playheadTimeOf(state, sequenceId));
}
