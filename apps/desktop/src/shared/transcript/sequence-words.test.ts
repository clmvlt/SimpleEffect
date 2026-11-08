import type { Flicks, Transcript } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import vectorsJson from '../../../../../docs/test-vectors/transcript-mapping.json?raw';
import { sequenceTranscript, wordAt, type SoundClip } from './sequence-words';

/**
 * Parity with Rust: the shared vectors also run in
 * `crates/se-domain/tests/transcript_mapping.rs`, so the panel and the agents' tools see the same
 * words at the same times.
 */

interface RawClip {
  readonly item: string;
  readonly asset: string;
  readonly startMs: number;
  readonly endMs: number;
  readonly sourceStartMs: number;
  readonly speed: number;
}

interface Vectors {
  readonly transcripts: Record<string, readonly [string, number, number, number][]>;
  readonly cases: readonly {
    readonly name: string;
    readonly tracks: readonly { readonly kind: string; readonly clips: readonly RawClip[] }[];
    readonly words: readonly [string, number, number, string, number, number][];
    readonly segments: readonly [string, number, number, number, number, number][];
  }[];
}

const vectors = JSON.parse(vectorsJson) as Vectors;
const ms = (value: number) => (value * 705_600) as Flicks;

function transcript(words: readonly [string, number, number, number][]): Transcript {
  return {
    language: 'fr',
    model: 'base',
    words: words.map(([text, start, end, segment]) => ({
      text,
      start: ms(start),
      end: ms(end),
      confidence: 0.9,
      segment,
    })),
    segments: [],
  };
}

const transcripts = new Map(
  Object.entries(vectors.transcripts).map(([asset, words]) => [asset, transcript(words)]),
);

describe('sequenceTranscript', () => {
  for (const vector of vectors.cases) {
    it(vector.name, () => {
      const clips: SoundClip[] = vector.tracks.flatMap((track, trackIndex) =>
        track.kind !== 'audio'
          ? []
          : track.clips.map((clip) => ({
              itemId: clip.item,
              assetId: clip.asset,
              start: ms(clip.startMs),
              duration: ms(clip.endMs - clip.startMs),
              sourceStart: ms(clip.sourceStartMs),
              speed: clip.speed,
              trackIndex,
            })),
      );
      const result = sequenceTranscript(clips, (asset) => transcripts.get(asset));
      expect(
        result.words.map((word) => [
          word.text,
          word.start,
          word.end,
          word.itemId,
          word.wordIndex,
          word.segment,
        ]),
      ).toEqual(
        vector.words.map(([text, start, end, item, index, segment]) => [
          text,
          ms(start),
          ms(end),
          item,
          index,
          segment,
        ]),
      );
      expect(
        result.segments.map((segment) => [
          segment.itemId,
          segment.start,
          segment.end,
          segment.firstWord,
          segment.wordCount,
          segment.sourceSegment,
        ]),
      ).toEqual(
        vector.segments.map(([item, start, end, first, count, source]) => [
          item,
          ms(start),
          ms(end),
          first,
          count,
          source,
        ]),
      );
    });
  }
});

describe('wordAt', () => {
  it('finds the word playing at a time, or none in a pause', () => {
    const clip: SoundClip = {
      itemId: 'c',
      assetId: 'a',
      start: ms(0),
      duration: ms(2000),
      sourceStart: ms(0),
      speed: 1,
      trackIndex: 0,
    };
    const words = sequenceTranscript([clip], () => transcripts.get('a')).words;
    expect(wordAt(words, ms(450))).toBe(1);
    expect(wordAt(words, ms(1200))).toBe(-1);
    expect(wordAt(words, ms(1999))).toBe(4);
    expect(wordAt([], ms(0))).toBe(-1);
  });
});
