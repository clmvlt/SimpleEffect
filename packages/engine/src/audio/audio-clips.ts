import type { Animatable, AssetId, Flicks, ItemId, TrackId } from '@se/protocol';

import type { EvaluableItem, EvaluableTrack } from '../evaluate/evaluate';
import type { MixTrack } from './mix';

/** A track the audio engine plays: its mixing state and its items. */
export interface PlayableTrack extends MixTrack, Pick<EvaluableTrack, 'hidden' | 'items'> {}

/** The parts of a sequence the audio engine reads (a domain `Sequence` fits). */
export interface PlayableSequence {
  readonly tracks: readonly PlayableTrack[];
}

/** A media clip on an audio track: the sound it plays and where. */
export interface AudioClip {
  readonly itemId: ItemId;
  readonly trackId: TrackId;
  readonly assetId: AssetId;
  /** Sequence time `[start, end)`. */
  readonly start: Flicks;
  readonly end: Flicks;
  /** Asset time played at `start`. */
  readonly sourceStart: Flicks;
  readonly speed: number;
  readonly volumeDb: Animatable<number>;
}

function clipOf(trackId: TrackId, item: EvaluableItem): AudioClip | null {
  if (item.kind.kind !== 'mediaClip') return null;
  const { start, duration } = item.timelineRange;
  return {
    itemId: item.id,
    trackId,
    assetId: item.kind.assetId,
    start,
    end: (start + duration) as Flicks,
    sourceStart: item.kind.sourceStart,
    speed: item.kind.speed,
    volumeDb: item.kind.volumeDb,
  };
}

/**
 * The clips whose sound plays (pure): media clips on audio tracks — a clip on a video track only
 * shows its picture. Muted tracks are kept (their strip is silent), so unmuting is immediate.
 */
export function audioClipsOf(sequence: PlayableSequence): AudioClip[] {
  const clips: AudioClip[] = [];
  for (const track of sequence.tracks) {
    if (track.kind !== 'audio') continue;
    for (const item of track.items) {
      const clip = clipOf(track.id, item);
      if (clip !== null) clips.push(clip);
    }
  }
  return clips;
}

function sameVolume(left: Animatable<number>, right: Animatable<number>): boolean {
  if (left === right) return true;
  if (left.kind === 'static' && right.kind === 'static') return left.value === right.value;
  return JSON.stringify(left) === JSON.stringify(right);
}

/** Whether two clip lists play the same sound (an unchanged edit keeps scheduled audio). */
export function sameClips(left: readonly AudioClip[], right: readonly AudioClip[]): boolean {
  return (
    left.length === right.length &&
    left.every((clip, index) => {
      const other = right[index];
      return (
        other !== undefined &&
        clip.itemId === other.itemId &&
        clip.trackId === other.trackId &&
        clip.assetId === other.assetId &&
        clip.start === other.start &&
        clip.end === other.end &&
        clip.sourceStart === other.sourceStart &&
        clip.speed === other.speed &&
        sameVolume(clip.volumeDb, other.volumeDb)
      );
    })
  );
}
