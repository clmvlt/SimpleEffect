import type { AssetId, SequenceId } from '@se/protocol';

import type { ProjectMirror } from '../project-store';
import type { SoundClip } from './sequence-words';

/**
 * The media clips of the sound tracks of `sequenceId`, in track order: what the sequence's
 * transcript is made of (picture clips say nothing; muted tracks still count).
 */
export function soundClipsOf(mirror: ProjectMirror, sequenceId: SequenceId): SoundClip[] {
  const sequence = mirror.sequences[sequenceId];
  if (sequence === undefined) return [];
  const clips: SoundClip[] = [];
  sequence.trackIds.forEach((trackId, trackIndex) => {
    const track = mirror.tracks[trackId];
    if (track?.kind !== 'audio') return;
    for (const itemId of track.itemIds) {
      const item = mirror.items[itemId];
      if (item?.kind.kind !== 'mediaClip') continue;
      clips.push({
        itemId: item.id,
        assetId: item.kind.assetId,
        start: item.timelineRange.start,
        duration: item.timelineRange.duration,
        sourceStart: item.kind.sourceStart,
        speed: item.kind.speed,
        trackIndex,
      });
    }
  });
  return clips;
}

/** The distinct assets of `clips`, in first-use order. */
export function assetsOf(clips: readonly SoundClip[]): AssetId[] {
  return [...new Set(clips.map((clip) => clip.assetId))];
}
