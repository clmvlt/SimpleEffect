import type { Animatable, AssetId, FadeShape, Flicks, ItemId, TrackId } from '@se/protocol';

import type { EvaluableItem, EvaluableTrack } from '../evaluate/evaluate';
import { playExtensions, trackTransitionSpans, type TransitionSpan } from '../evaluate/transitions';
import type { MixTrack } from './mix';

/** A track the audio engine plays: its mixing state, its items and its cross-fades. */
export interface PlayableTrack
  extends MixTrack, Pick<EvaluableTrack, 'hidden' | 'items' | 'transitions'> {}

/** The parts of a sequence the audio engine reads (a domain `Sequence` fits). */
export interface PlayableSequence {
  readonly tracks: readonly PlayableTrack[];
}

/** A stretch of sequence time `[start, end)`, in flicks. */
export interface TimeSpan {
  readonly start: number;
  readonly end: number;
}

/** A media clip on an audio track: the sound it plays, where and how loud. */
export interface AudioClip {
  readonly itemId: ItemId;
  readonly trackId: TrackId;
  readonly assetId: AssetId;
  /** Sequence time of the item `[start, end)`: its fades and volume keyframes are relative to it. */
  readonly start: Flicks;
  readonly end: Flicks;
  /**
   * Sequence time its sound plays `[playStart, playEnd)`: the item, extended through its
   * cross-fades (before its start when it comes in, past its end when it goes out).
   */
  readonly playStart: Flicks;
  readonly playEnd: Flicks;
  /** Asset time played at `start`. */
  readonly sourceStart: Flicks;
  readonly speed: number;
  readonly volumeDb: Animatable<number>;
  readonly fadeIn: Flicks;
  readonly fadeOut: Flicks;
  readonly fadeInShape: FadeShape;
  readonly fadeOutShape: FadeShape;
  /** The cross-fade bringing the clip in (constant power), if any. */
  readonly crossfadeIn: TimeSpan | null;
  /** The cross-fade taking it out, if any. */
  readonly crossfadeOut: TimeSpan | null;
}

function spanOf(span: TransitionSpan | undefined): TimeSpan | null {
  return span === undefined ? null : { start: span.start, end: span.end };
}

function clipOf(
  trackId: TrackId,
  item: EvaluableItem,
  spans: readonly TransitionSpan[],
): AudioClip | null {
  if (item.kind.kind !== 'mediaClip') return null;
  const { start, duration } = item.timelineRange;
  const extension = playExtensions(spans).get(item.id);
  const clip = item.kind;
  return {
    itemId: item.id,
    trackId,
    assetId: clip.assetId,
    start,
    end: (start + duration) as Flicks,
    playStart: (start - (extension?.before ?? 0)) as Flicks,
    playEnd: (start + duration + (extension?.after ?? 0)) as Flicks,
    sourceStart: clip.sourceStart,
    speed: clip.speed,
    volumeDb: clip.volumeDb,
    fadeIn: clip.fadeIn,
    fadeOut: clip.fadeOut,
    fadeInShape: clip.fadeInShape,
    fadeOutShape: clip.fadeOutShape,
    crossfadeIn: spanOf(spans.find((span) => span.rightItem === item.id)),
    crossfadeOut: spanOf(spans.find((span) => span.leftItem === item.id)),
  };
}

/**
 * The clips whose sound plays (pure): media clips on audio tracks — a clip on a video track only
 * shows its picture — with their sound cross-fades. Muted tracks are kept (their strip is
 * silent), so unmuting is immediate.
 */
export function audioClipsOf(sequence: PlayableSequence): AudioClip[] {
  const clips: AudioClip[] = [];
  for (const track of sequence.tracks) {
    if (track.kind !== 'audio') continue;
    const crossfades = (track.transitions ?? []).filter(
      (transition) => transition.kind === 'audioCrossfade',
    );
    const spans = trackTransitionSpans(track.items, crossfades);
    for (const item of track.items) {
      const clip = clipOf(track.id, item, spans);
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

function sameSpan(left: TimeSpan | null, right: TimeSpan | null): boolean {
  return left === right || (left?.start === right?.start && left?.end === right?.end);
}

/** Whether two clips play the same sound at the same levels. */
function sameClip(clip: AudioClip, other: AudioClip): boolean {
  return (
    clip.itemId === other.itemId &&
    clip.trackId === other.trackId &&
    clip.assetId === other.assetId &&
    clip.start === other.start &&
    clip.end === other.end &&
    clip.playStart === other.playStart &&
    clip.playEnd === other.playEnd &&
    clip.sourceStart === other.sourceStart &&
    clip.speed === other.speed &&
    clip.fadeIn === other.fadeIn &&
    clip.fadeOut === other.fadeOut &&
    clip.fadeInShape === other.fadeInShape &&
    clip.fadeOutShape === other.fadeOutShape &&
    sameSpan(clip.crossfadeIn, other.crossfadeIn) &&
    sameSpan(clip.crossfadeOut, other.crossfadeOut) &&
    sameVolume(clip.volumeDb, other.volumeDb)
  );
}

/** Whether two clip lists play the same sound (an unchanged edit keeps scheduled audio). */
export function sameClips(left: readonly AudioClip[], right: readonly AudioClip[]): boolean {
  return (
    left.length === right.length &&
    left.every((clip, index) => {
      const other = right[index];
      return other !== undefined && sameClip(clip, other);
    })
  );
}
