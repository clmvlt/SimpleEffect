import type { AssetId, Flicks, ItemId, Transcript } from '@se/protocol';

import {
  EMPTY_SEQUENCE_TRANSCRIPT,
  type SequenceSegment,
  type SequenceTranscript,
  type SequenceWord,
} from '../../../shared/transcript';

/** The item id given to words shown in their media's own time (no clip plays them here). */
const NO_ITEM = '' as ItemId;

/**
 * One asset's transcript in its own (source) time, shaped like a sequence transcript so the panel
 * renders both alike: one segment per transcript segment.
 */
export function mediaTranscript(
  assetId: AssetId,
  transcript: Transcript | undefined,
): SequenceTranscript {
  if (transcript === undefined) return EMPTY_SEQUENCE_TRANSCRIPT;
  const words: SequenceWord[] = [];
  const segments: SequenceSegment[] = [];
  transcript.words.forEach((word, index) => {
    const last = segments.at(-1);
    if (last?.sourceSegment !== word.segment) {
      segments.push({
        start: word.start,
        end: word.end,
        itemId: NO_ITEM,
        assetId,
        firstWord: index,
        wordCount: 0,
        sourceSegment: word.segment,
      });
    }
    const current = segments[segments.length - 1];
    if (current !== undefined) {
      segments[segments.length - 1] = {
        ...current,
        end: Math.max(current.end, word.end) as Flicks,
        wordCount: current.wordCount + 1,
      };
    }
    words.push({
      text: word.text,
      start: word.start,
      end: word.end,
      confidence: word.confidence,
      assetId,
      itemId: NO_ITEM,
      wordIndex: index,
      segment: segments.length - 1,
    });
  });
  return { words, segments };
}

/**
 * Where the sequence plays word `wordIndex` of `assetId`: its first occurrence in `sequenceWords`,
 * or `null` when the sequence does not play it (cut or unused).
 */
export function sequenceTimeOf(
  sequenceWords: SequenceTranscript,
  assetId: AssetId,
  wordIndex: number,
): Flicks | null {
  const found = sequenceWords.words.find(
    (word) => word.assetId === assetId && word.wordIndex === wordIndex,
  );
  return found?.start ?? null;
}
