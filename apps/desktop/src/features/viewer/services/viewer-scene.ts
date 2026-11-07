import type {
  Asset,
  AssetId,
  ItemEntity,
  ItemId,
  SequenceEntity,
  TrackEntity,
  TrackId,
} from '@se/protocol';
import type { EvaluableItem, EvaluableTrack } from '@se/engine';
import type { RenderScene } from '@se/engine/preview';

/** The mirror entities a scene is built from (the read-only project store's records). */
export interface SceneSource {
  readonly sequence: SequenceEntity;
  readonly tracks: Readonly<Record<TrackId, TrackEntity>>;
  readonly items: Readonly<Record<ItemId, ItemEntity>>;
  readonly assets: Readonly<Record<AssetId, Asset>>;
}

/**
 * The scene the engine draws for a sequence of the mirror: its tracks in stacking order with
 * their items (entities are kept as they are: they already have the fields `evaluate` reads).
 */
export function sceneOf({ sequence, tracks, items, assets }: SceneSource): RenderScene {
  const sequenceTracks: EvaluableTrack[] = [];
  for (const trackId of sequence.trackIds) {
    const track = tracks[trackId];
    if (track === undefined) continue;
    const trackItems: EvaluableItem[] = [];
    for (const itemId of track.itemIds) {
      const item = items[itemId];
      if (item !== undefined) trackItems.push(item);
    }
    sequenceTracks.push({ kind: track.kind, hidden: track.hidden, items: trackItems });
  }
  return { sequence: { format: sequence.format, tracks: sequenceTracks }, assets };
}
