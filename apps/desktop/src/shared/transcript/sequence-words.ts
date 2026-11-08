import type { AssetId, Flicks, ItemId, Transcript } from '@se/protocol';

import { clampFlicks } from '../time';

/**
 * The words of a sequence in sequence time: mirror of `se_domain::sequence_transcript` (prompt
 * 21), tested on the same vectors (`docs/test-vectors/transcript-mapping.json`). Change both
 * sides together.
 *
 * Rules: only clips of sound tracks count; a word belongs to the clip playing its middle; times go
 * through the clip's speed and are clamped to the clip; one word of one asset at one sequence time
 * appears once; words are sorted by start, then track order; consecutive words of one clip and
 * one source segment form a segment.
 */

/** A media clip of a sound track, as the mapping needs it. */
export interface SoundClip {
  readonly itemId: ItemId;
  readonly assetId: AssetId;
  /** Start of the item in the sequence. */
  readonly start: Flicks;
  /** Duration of the item in the sequence. */
  readonly duration: Flicks;
  /** Where it starts in its asset. */
  readonly sourceStart: Flicks;
  /** Source seconds per sequence second. */
  readonly speed: number;
  /** Order of its track in the sequence (ties between words at one time). */
  readonly trackIndex: number;
}

/** A word in sequence time. */
export interface SequenceWord {
  readonly text: string;
  readonly start: Flicks;
  readonly end: Flicks;
  /** 0–1. */
  readonly confidence: number;
  readonly assetId: AssetId;
  readonly itemId: ItemId;
  /** Index in the asset's transcript. */
  readonly wordIndex: number;
  /** Index in {@link SequenceTranscript.segments}. */
  readonly segment: number;
}

/** Consecutive words of one clip and one source segment. */
export interface SequenceSegment {
  readonly start: Flicks;
  readonly end: Flicks;
  readonly itemId: ItemId;
  readonly assetId: AssetId;
  readonly firstWord: number;
  readonly wordCount: number;
  /** The segment of the asset's transcript its words come from. */
  readonly sourceSegment: number;
}

/** The words of a sequence. */
export interface SequenceTranscript {
  readonly words: readonly SequenceWord[];
  readonly segments: readonly SequenceSegment[];
}

/** The empty transcript. */
export const EMPTY_SEQUENCE_TRANSCRIPT: SequenceTranscript = { words: [], segments: [] };

interface Mapped {
  readonly word: Omit<SequenceWord, 'segment'>;
  readonly trackIndex: number;
  readonly sourceSegment: number;
}

/** The middle of `[start, end]`, rounded toward the start (Rust's integer division). */
function middle(start: Flicks, end: Flicks): Flicks {
  return (start + Math.trunc((end - start) / 2)) as Flicks;
}

/** The words of `clips` whose assets have a transcript (`transcriptOf`), in sequence time. */
export function sequenceTranscript(
  clips: readonly SoundClip[],
  transcriptOf: (assetId: AssetId) => Transcript | undefined,
): SequenceTranscript {
  const mapped: Mapped[] = [];
  for (const clip of clips) {
    const transcript = transcriptOf(clip.assetId);
    if (transcript !== undefined && clip.speed > 0) collect(clip, transcript, mapped);
  }
  const seen = new Set<string>();
  const unique = mapped.filter((entry) => {
    const key = `${entry.word.assetId}:${String(entry.word.wordIndex)}:${String(entry.word.start)}`;
    if (seen.has(key)) return false;
    seen.add(key);
    return true;
  });
  unique.sort(
    (left, right) =>
      left.word.start - right.word.start ||
      left.trackIndex - right.trackIndex ||
      left.word.wordIndex - right.word.wordIndex,
  );
  return group(unique);
}

/** Pushes the words whose middle `clip` plays. */
function collect(clip: SoundClip, transcript: Transcript, out: Mapped[]): void {
  const sourceEnd = clip.sourceStart + clampFlicks(clip.duration * clip.speed);
  const timelineEnd = clip.start + clip.duration;
  const toTimeline = (time: Flicks): Flicks =>
    (clip.start + clampFlicks((time - clip.sourceStart) * (1 / clip.speed))) as Flicks;
  const words = transcript.words;
  let index = firstIndexFrom(words, clip.sourceStart);
  for (; index < words.length; index += 1) {
    const word = words[index];
    if (word === undefined || middle(word.start, word.end) >= sourceEnd) break;
    const start = Math.max(toTimeline(word.start), clip.start) as Flicks;
    const end = Math.min(toTimeline(word.end), timelineEnd) as Flicks;
    out.push({
      word: {
        text: word.text,
        start,
        end: Math.max(end, start) as Flicks,
        confidence: word.confidence,
        assetId: clip.assetId,
        itemId: clip.itemId,
        wordIndex: index,
      },
      trackIndex: clip.trackIndex,
      sourceSegment: word.segment,
    });
  }
}

/** Index of the first word whose middle is at or after `time` (words sorted by start). */
function firstIndexFrom(words: Transcript['words'], time: Flicks): number {
  let low = 0;
  let high = words.length;
  while (low < high) {
    const mid = (low + high) >>> 1;
    const word = words[mid];
    if (word !== undefined && middle(word.start, word.end) < time) low = mid + 1;
    else high = mid;
  }
  return low;
}

/** Numbers segments: a new one whenever the clip or the source segment changes. */
function group(mapped: readonly Mapped[]): SequenceTranscript {
  const words: SequenceWord[] = [];
  const segments: SequenceSegment[] = [];
  let previous: string | null = null;
  for (const { word, sourceSegment } of mapped) {
    const key = `${word.itemId}:${String(sourceSegment)}`;
    const last = segments.at(-1);
    if (key !== previous || last === undefined) {
      segments.push({
        start: word.start,
        end: word.end,
        itemId: word.itemId,
        assetId: word.assetId,
        firstWord: words.length,
        wordCount: 1,
        sourceSegment,
      });
      previous = key;
    } else {
      segments[segments.length - 1] = {
        ...last,
        end: Math.max(last.end, word.end) as Flicks,
        wordCount: last.wordCount + 1,
      };
    }
    words.push({ ...word, segment: segments.length - 1 });
  }
  return { words, segments };
}

/** Index of the word playing at `time` (its range holds it), or `-1` (words sorted by start). */
export function wordAt(words: readonly SequenceWord[], time: Flicks): number {
  let low = 0;
  let high = words.length - 1;
  let found = -1;
  while (low <= high) {
    const mid = (low + high) >>> 1;
    const word = words[mid];
    if (word === undefined) break;
    if (word.start <= time) {
      found = mid;
      low = mid + 1;
    } else {
      high = mid - 1;
    }
  }
  const word = found >= 0 ? words[found] : undefined;
  return word !== undefined && time < word.end ? found : -1;
}
