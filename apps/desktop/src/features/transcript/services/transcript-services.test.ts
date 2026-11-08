import type { Flicks, Transcript } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { mediaTranscript, sequenceTimeOf } from './media-words';
import { cleanVocabulary, languageName, segmentClock } from './transcript-format';
import { findMatches, foldText, stepMatch } from './word-search';

const words = (...texts: string[]) => texts.map((text) => ({ text }));
const ms = (value: number) => (value * 705_600) as Flicks;

describe('word search', () => {
  it('folds accents, case and punctuation', () => {
    expect(foldText('Abonnez-vous à la Chaîne !')).toBe('abonnez vous a la chaine');
    expect(foldText("L'Été")).toBe('l ete');
  });

  it('finds consecutive words, the last one as a prefix', () => {
    const text = words('Bonjour', 'à', 'tous.', 'Abonnez-vous', 'à', 'la', 'chaîne !');
    expect(findMatches(text, 'abonn')).toEqual([{ first: 3, last: 3 }]);
    expect(findMatches(text, 'vous a la CHAI')).toEqual([{ first: 3, last: 6 }]);
    // The last word is a prefix: « à » also finds « Abonnez-vous ».
    expect(findMatches(text, 'à')).toEqual([
      { first: 1, last: 1 },
      { first: 3, last: 3 },
      { first: 4, last: 4 },
    ]);
    expect(findMatches(text, '  ?! ')).toEqual([]);
    expect(findMatches(text, 'tous abonnez')).toEqual([{ first: 2, last: 3 }]);
  });

  it('steps through matches in both directions, wrapping around', () => {
    expect(stepMatch(3, null, 1)).toBe(0);
    expect(stepMatch(3, null, -1)).toBe(2);
    expect(stepMatch(3, 2, 1)).toBe(0);
    expect(stepMatch(3, 0, -1)).toBe(2);
    expect(stepMatch(0, null, 1)).toBeNull();
  });
});

describe('transcript format', () => {
  it('labels segments and languages', () => {
    expect(segmentClock(ms(65_400))).toBe('1:05');
    expect(segmentClock(ms(3_725_000))).toBe('1:02:05');
    expect(languageName('fr')).toBe('français');
    expect(cleanVocabulary('  SimpleEffect,\n Dimzou  ')).toBe('SimpleEffect, Dimzou');
  });
});

describe('media words', () => {
  const transcript: Transcript = {
    language: 'fr',
    model: 'base',
    words: [
      { text: 'Bonjour', start: ms(0), end: ms(400), confidence: 0.9, segment: 0 },
      { text: 'tous.', start: ms(400), end: ms(800), confidence: 0.9, segment: 0 },
      { text: 'Ça', start: ms(1500), end: ms(1700), confidence: 0.2, segment: 1 },
    ],
    segments: [],
  };

  it('shows a media transcript in its own time, one paragraph per segment', () => {
    const view = mediaTranscript('a', transcript);
    expect(view.segments.map((segment) => [segment.firstWord, segment.wordCount])).toEqual([
      [0, 2],
      [2, 1],
    ]);
    expect(view.words[2]?.segment).toBe(1);
    expect(mediaTranscript('a', undefined).words).toEqual([]);
  });

  it('finds where the sequence plays a media word', () => {
    const media = mediaTranscript('a', transcript);
    const played = {
      ...media,
      words: media.words.map((word) => ({ ...word, start: (word.start + ms(10_000)) as Flicks })),
    };
    expect(sequenceTimeOf(played, 'a', 1)).toBe(ms(10_400));
    expect(sequenceTimeOf(played, 'b', 1)).toBeNull();
  });
});
