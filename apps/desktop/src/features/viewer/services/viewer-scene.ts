import type {
  Asset,
  AssetId,
  Flicks,
  ItemEntity,
  ItemId,
  SequenceEntity,
  TrackEntity,
  TrackId,
} from '@se/protocol';
import type { EvaluableItem } from '@se/engine';
import type { AudioScene, PlayableTrack } from '@se/engine/audio';
import type { RenderScene } from '@se/engine/preview';

/**
 * What the viewer plays: a scene the picture engine draws and the audio engine plays (its tracks
 * carry their mixing state).
 */
export interface PlaybackScene extends RenderScene, AudioScene {
  readonly sequence: RenderScene['sequence'] &
    AudioScene['sequence'] & {
      readonly tracks: readonly PlayableTrack[];
    };
}

/** The mirror entities a scene is built from (the read-only project store's records). */
export interface SceneSource {
  readonly sequence: SequenceEntity;
  readonly tracks: Readonly<Record<TrackId, TrackEntity>>;
  readonly items: Readonly<Record<ItemId, ItemEntity>>;
  readonly assets: Readonly<Record<AssetId, Asset>>;
}

/**
 * The scene the engines play for a sequence of the mirror: its tracks in stacking order with
 * their mixing state and items (entities are kept as they are: they already have the fields
 * `evaluate` and the audio engine read).
 */
export function sceneOf({ sequence, tracks, items, assets }: SceneSource): PlaybackScene {
  const sequenceTracks: PlayableTrack[] = [];
  for (const trackId of sequence.trackIds) {
    const track = tracks[trackId];
    if (track === undefined) continue;
    const trackItems: EvaluableItem[] = [];
    for (const itemId of track.itemIds) {
      const item = items[itemId];
      if (item !== undefined) trackItems.push(item);
    }
    const { id, kind, hidden, muted, solo, volumeDb, pan } = track;
    sequenceTracks.push({ id, kind, hidden, muted, solo, volumeDb, pan, items: trackItems });
  }
  return { sequence: { format: sequence.format, tracks: sequenceTracks }, assets };
}

/** End of the last item of `scene` (where playback stops), zero when empty. */
export function sceneContentEnd(scene: PlaybackScene): Flicks {
  let end = 0;
  for (const track of scene.sequence.tracks) {
    const last = track.items.at(-1);
    if (last !== undefined)
      end = Math.max(end, last.timelineRange.start + last.timelineRange.duration);
  }
  return end as Flicks;
}
