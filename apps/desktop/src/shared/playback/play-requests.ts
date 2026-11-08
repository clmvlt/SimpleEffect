import type { Flicks, SequenceId } from '@se/protocol';

/** A passage another feature wants heard (the transcript's « écouter » of a filler, prompt 22). */
export interface PlayRequest {
  readonly sequenceId: SequenceId;
  /** Where playback starts. */
  readonly start: Flicks;
  /** Where it pauses. */
  readonly end: Flicks;
}

type Listener = (request: PlayRequest) => void;

const listeners = new Set<Listener>();

/** Asks the viewer to play `start`–`end` of `sequenceId`, then pause. */
export function requestPlayRange(request: PlayRequest): void {
  for (const listener of listeners) listener(request);
}

/** Receives play requests (the viewer's playback session); returns the unsubscribe function. */
export function onPlayRequest(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
