import type { Asset, Flicks, TrackEntity, TrackTarget } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import { fileName, type ItemTone } from './item-display';
import type { GhostRow, PreviewGhost } from './preview';
import { trackNumbers, tracksByKind } from './track-numbers';

/**
 * How long a dropped still image lasts. Same value as `DEFAULT_IMAGE_DURATION` in
 * `crates/se-app/src/edit_plan/new_items.rs`: change both together.
 */
export const DEFAULT_IMAGE_DURATION = (5 * FLICKS_PER_SECOND) as Flicks;

/** The tracks the picture and the sound of dropped media go to. */
export interface DropTargets {
  readonly video: TrackTarget;
  readonly audio: TrackTarget;
}

const target = (track: TrackEntity | undefined): TrackTarget =>
  track === undefined ? { kind: 'new' } : { kind: 'existing', trackId: track.id };

/**
 * Where media dropped over `row` goes: the row's track for the parts of its kind, the track of
 * the other kind with the same number for the others (V2 ↔ A2), a new track when it does not
 * exist. Below the last track (`row` null), sound goes to a new audio track.
 */
export function dropTargets(tracks: readonly TrackEntity[], row: TrackEntity | null): DropTargets {
  const byKind = tracksByKind(tracks);
  if (row === null) {
    return { video: target(byKind.video[byKind.audio.length]), audio: { kind: 'new' } };
  }
  const number = trackNumbers(tracks).get(row.id) ?? 0;
  return row.kind === 'video'
    ? { video: target(row), audio: target(byKind.audio[number]) }
    : { video: target(byKind.video[number]), audio: target(row) };
}

/** Which parts of an asset a drop places and how long they last (`null`: unknown, not placed). */
export function assetParts(asset: Asset): {
  readonly video: boolean;
  readonly audio: boolean;
  readonly duration: Flicks | null;
} {
  const duration = asset.mediaInfo?.duration ?? null;
  const sound = (asset.mediaInfo?.audioStreams.length ?? 0) > 0;
  switch (asset.kind) {
    case 'image':
      return { video: true, audio: false, duration: DEFAULT_IMAGE_DURATION };
    case 'video':
      return { video: true, audio: sound, duration };
    case 'audio':
      return { video: false, audio: true, duration };
  }
}

/** Where the ghost of a part lands: an existing track, the new audio row, or nowhere drawn. */
function ghostRow(target: TrackTarget, kind: 'video' | 'audio'): GhostRow | null {
  if (target.kind === 'existing') return { kind: 'track', trackId: target.trackId };
  return kind === 'audio' ? { kind: 'newAudio' } : null;
}

/** The ghosts of `assets` dropped at `start`, one after the other, as the core will place them. */
export function dropGhosts(
  assets: readonly Asset[],
  start: Flicks,
  targets: DropTargets,
): PreviewGhost[] {
  const ghosts: PreviewGhost[] = [];
  let cursor: number = start;
  for (const asset of assets) {
    const parts = assetParts(asset);
    if (parts.duration === null) continue;
    const label = fileName(asset.originalPath);
    const add = (kind: 'video' | 'audio', tone: ItemTone) => {
      const row = ghostRow(targets[kind], kind);
      if (row === null || parts.duration === null) return;
      ghosts.push({
        key: `${asset.id}-${kind}-${cursor.toString()}`,
        row,
        start: cursor as Flicks,
        duration: parts.duration,
        source: { kind: 'media', label, tone },
      });
    };
    if (parts.video) add('video', 'video');
    if (parts.audio) add('audio', 'audio');
    cursor += parts.duration;
  }
  return ghosts;
}
