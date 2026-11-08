import type { AssetId, Flicks, ItemId, ItemTrim, Transcript, TranscriptWord } from '@se/protocol';

import type { ProjectMirror } from '../../../shared/project-store';
import type { SequenceWord, SoundClip } from '../../../shared/transcript';

/**
 * Passages cut out of the sequence (prompt 22): where two clips of one asset meet on a sound
 * track with a jump forward in the source, the source between them is what a cut removed. Its
 * words are shown struck through at that place (« afficher les passages coupés »); a passage
 * without words is a silence that was cut (« ⋯ 1,2 s »). Only junctions are known: a cut at the
 * very start or end of a clip leaves no neighbour to compare with.
 */
export interface CutPassage {
  /** Where the two clips meet in the sequence. */
  readonly at: Flicks;
  /** The clip ending at `at` and the one starting there. */
  readonly leftItemId: ItemId;
  readonly rightItemId: ItemId;
  readonly assetId: AssetId;
  /** The words of the removed source (empty for a silence). */
  readonly words: readonly TranscriptWord[];
  /** How long the passage lasted in the sequence (source time over the clips' speed). */
  readonly duration: Flicks;
  /** Index of the sequence word it shows before (`words.length` at the very end). */
  readonly beforeWord: number;
}

/** Two clips this close (one millisecond) are taken as touching. */
const TOUCHING = 705_600;

/** Shortest silence worth a mark (shorter ones are frame rounding). */
const MIN_SILENCE_MARK = 70_560_000;

function sourceEnd(clip: SoundClip): number {
  return clip.sourceStart + clip.duration * clip.speed;
}

function middle(word: TranscriptWord): number {
  return word.start + Math.trunc((word.end - word.start) / 2);
}

/** The index of the first word starting at or after `time`. */
function firstWordFrom(words: readonly SequenceWord[], time: number): number {
  let low = 0;
  let high = words.length;
  while (low < high) {
    const mid = (low + high) >> 1;
    if ((words[mid]?.start ?? Infinity) < time) low = mid + 1;
    else high = mid;
  }
  return low;
}

/**
 * The passages cut between touching clips of `clips` (sound clips, any order), with the words of
 * each asset's transcript (`transcriptOf`) placed among the sequence's `words`.
 */
export function cutPassages(
  clips: readonly SoundClip[],
  transcriptOf: (assetId: AssetId) => Transcript | undefined,
  words: readonly SequenceWord[],
): CutPassage[] {
  const byTrack = new Map<number, SoundClip[]>();
  for (const clip of clips)
    byTrack.set(clip.trackIndex, [...(byTrack.get(clip.trackIndex) ?? []), clip]);
  const passages: CutPassage[] = [];
  const seen = new Set<string>();
  for (const trackClips of byTrack.values()) {
    const sorted = [...trackClips].sort((a, b) => a.start - b.start);
    sorted.forEach((left, index) => {
      const right = sorted[index + 1];
      if (right === undefined) return;
      const at = left.start + left.duration;
      const removed = right.sourceStart - sourceEnd(left);
      const joined =
        Math.abs(right.start - at) <= TOUCHING &&
        right.assetId === left.assetId &&
        right.speed === left.speed &&
        removed > 0;
      if (!joined) return;
      const key = `${left.assetId}:${String(at)}:${String(right.sourceStart)}`;
      if (seen.has(key)) return;
      seen.add(key);
      const from = sourceEnd(left);
      const removedWords = (transcriptOf(left.assetId)?.words ?? []).filter(
        (word) => middle(word) >= from && middle(word) < right.sourceStart,
      );
      const duration = Math.round(removed / left.speed);
      if (removedWords.length === 0 && duration < MIN_SILENCE_MARK) return;
      passages.push({
        at: at as Flicks,
        leftItemId: left.itemId,
        rightItemId: right.itemId,
        assetId: left.assetId,
        words: removedWords,
        duration: duration as Flicks,
        beforeWord: firstWordFrom(words, at),
      });
    });
  }
  return passages.sort((a, b) => a.at - b.at);
}

/**
 * The ripple trims putting `passage` back: the left clip and its linked partners ending at the
 * same instant get their end extended by the passage, so what follows moves right and the source
 * plays again where it was cut. `null` when the left clip is gone from `mirror`.
 */
export function restoreTrims(passage: CutPassage, mirror: ProjectMirror): ItemTrim[] | null {
  const left = mirror.items[passage.leftItemId];
  if (left === undefined) return null;
  const end = left.timelineRange.start + left.timelineRange.duration;
  const to = (end + passage.duration) as Flicks;
  const partners = Object.values(mirror.items).filter(
    (item) =>
      item.id !== left.id &&
      left.linkGroup !== null &&
      item.linkGroup === left.linkGroup &&
      item.timelineRange.start + item.timelineRange.duration === end,
  );
  return [left, ...partners].map((item) => ({ itemId: item.id, edge: 'end', to }));
}
