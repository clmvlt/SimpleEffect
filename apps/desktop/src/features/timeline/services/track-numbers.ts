import type { TrackEntity, TrackId, TrackKind } from '@se/protocol';

/** The tracks of each kind in document order (V1 first, A1 first). */
export interface TracksByKind {
  readonly video: readonly TrackEntity[];
  readonly audio: readonly TrackEntity[];
}

/** `tracks` split by kind, document order kept. */
export function tracksByKind(tracks: readonly TrackEntity[]): TracksByKind {
  return {
    video: tracks.filter((track) => track.kind === 'video'),
    audio: tracks.filter((track) => track.kind === 'audio'),
  };
}

/**
 * The number of each track among the tracks of its kind (V1 = 0, V2 = 1, A1 = 0…): gestures move
 * items by numbers, so a clip's picture going from V1 to V2 takes its sound from A1 to A2.
 */
export function trackNumbers(tracks: readonly TrackEntity[]): ReadonlyMap<TrackId, number> {
  const { video, audio } = tracksByKind(tracks);
  return new Map([
    ...video.map((track, index) => [track.id, index] as const),
    ...audio.map((track, index) => [track.id, index] as const),
  ]);
}

/** The track of `kind` numbered `number`, if the sequence has it. */
export function trackOfNumber(
  byKind: TracksByKind,
  kind: TrackKind,
  number: number,
): TrackEntity | undefined {
  return byKind[kind][number];
}
