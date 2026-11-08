import type { Flicks, SequenceId, TimeRange, TrackId } from '@se/protocol';

import type { ProjectMirror } from '../../../shared/project-store';

/** A sound track the silence panel can listen to. */
export interface ListenableTrack {
  readonly id: TrackId;
  readonly name: string;
  /** Listened to by default: unmuted, unlocked and holding media (as the core decides). */
  readonly byDefault: boolean;
}

/** The sound tracks of `sequenceId`, in sequence order. */
export function soundTracksOf(mirror: ProjectMirror, sequenceId: SequenceId): ListenableTrack[] {
  const sequence = mirror.sequences[sequenceId];
  if (sequence === undefined) return [];
  return sequence.trackIds.flatMap((id) => {
    const track = mirror.tracks[id];
    if (track?.kind !== 'audio') return [];
    const holdsMedia = track.itemIds.some(
      (itemId) => mirror.items[itemId]?.kind.kind === 'mediaClip',
    );
    return [{ id, name: track.name, byDefault: !track.muted && !track.locked && holdsMedia }];
  });
}

/** Where the content of `sequenceId` ends. */
export function sequenceEnd(mirror: ProjectMirror, sequenceId: SequenceId): Flicks {
  const sequence = mirror.sequences[sequenceId];
  let end = 0;
  for (const trackId of sequence?.trackIds ?? []) {
    for (const itemId of mirror.tracks[trackId]?.itemIds ?? []) {
      const range = mirror.items[itemId]?.timelineRange;
      if (range !== undefined) end = Math.max(end, range.start + range.duration);
    }
  }
  return end as Flicks;
}

/** The in/out span as a range (to `end` without an out point); `null` without an in point. */
export function inOutRange(
  inPoint: Flicks | null,
  outPoint: Flicks | null,
  end: Flicks,
): TimeRange | null {
  if (inPoint === null) return null;
  const stop = outPoint ?? end;
  return stop > inPoint ? { start: inPoint, duration: (stop - inPoint) as Flicks } : null;
}

/**
 * The tracks to send: `null` (the core's default) while `chosen` is exactly the default set, so
 * the preview and the cut follow the sequence when tracks change.
 */
export function listenedTracks(
  tracks: readonly ListenableTrack[],
  chosen: ReadonlySet<TrackId> | null,
): TrackId[] | null {
  if (chosen === null) return null;
  const defaults = tracks.filter((track) => track.byDefault).map((track) => track.id);
  const same = defaults.length === chosen.size && defaults.every((id) => chosen.has(id));
  return same ? null : tracks.filter((track) => chosen.has(track.id)).map((track) => track.id);
}
