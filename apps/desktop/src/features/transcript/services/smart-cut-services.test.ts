import type { Transcript, TrackEntity } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import type { ProjectMirror } from '../../../shared/project-store';
import type { SequenceWord, SoundClip } from '../../../shared/transcript';
import { seconds } from '../testing/fixtures';
import { cutSeconds, passageExcerpt } from './cut-format';
import { cutPassages, restoreTrims } from './cut-passages';
import { editedLanguages, parseWords, withTexts } from './filler-lists';
import { inOutRange, listenedTracks, soundTracksOf } from './silence-options';

const TRANSCRIPT: Transcript = {
  language: 'fr',
  model: 'base',
  words: [
    { text: 'Bonjour', start: seconds(0), end: seconds(0.4), confidence: 0.9, segment: 0 },
    { text: 'à', start: seconds(0.4), end: seconds(0.5), confidence: 0.9, segment: 0 },
    { text: 'tous.', start: seconds(0.5), end: seconds(0.9), confidence: 0.9, segment: 0 },
    { text: 'Voilà', start: seconds(3), end: seconds(3.4), confidence: 0.9, segment: 1 },
  ],
  segments: [],
};

function clip(itemId: string, start: number, length: number, source: number, track = 0): SoundClip {
  return {
    itemId,
    assetId: 'asset-1',
    start: seconds(start),
    duration: seconds(length),
    sourceStart: seconds(source),
    speed: 1,
    trackIndex: track,
  };
}

const WORDS: SequenceWord[] = [
  {
    text: 'Voilà',
    start: seconds(10.5),
    end: seconds(10.9),
    confidence: 0.9,
    assetId: 'asset-1',
    itemId: 'b',
    wordIndex: 3,
    segment: 0,
  },
];

describe('cutPassages', () => {
  it('finds the words and silences removed between touching clips of one asset', () => {
    const clips = [clip('a', 10, 0.4, 0), clip('b', 10.4, 1, 0.9), clip('c', 11.4, 1, 2.8)];
    const passages = cutPassages(clips, () => TRANSCRIPT, WORDS);
    expect(passages).toHaveLength(2);
    expect(passages[0]?.words.map((word) => word.text)).toEqual(['à', 'tous.']);
    expect(passages[0]?.beforeWord).toBe(0);
    expect(passages[0]?.duration).toBe(seconds(0.5));
    expect(passages[1]?.words).toEqual([]);
    expect(passages[1]?.duration).toBe(seconds(0.9));
  });

  it('ignores gaps, other assets, backward jumps and tiny silences, once per junction', () => {
    const apart = [clip('a', 10, 0.4, 0), clip('b', 11, 1, 0.9)];
    expect(cutPassages(apart, () => TRANSCRIPT, WORDS)).toEqual([]);
    const backward = [clip('a', 10, 1, 5), clip('b', 11, 1, 0)];
    expect(cutPassages(backward, () => TRANSCRIPT, WORDS)).toEqual([]);
    const tiny = [clip('a', 10, 1, 5), clip('b', 11, 1, 6.05)];
    expect(cutPassages(tiny, () => TRANSCRIPT, WORDS)).toEqual([]);
    const twoTracks = [
      clip('a', 10, 0.4, 0),
      clip('b', 10.4, 1, 0.9),
      clip('c', 10, 0.4, 0, 1),
      clip('d', 10.4, 1, 0.9, 1),
    ];
    expect(cutPassages(twoTracks, () => TRANSCRIPT, WORDS)).toHaveLength(1);
  });
});

describe('restoreTrims', () => {
  it('extends the clip before the cut and its linked partners ending with it', () => {
    const item = (id: string, trackId: string, linkGroup: string | null) => ({
      id,
      trackId,
      linkGroup,
      timelineRange: { start: seconds(10), duration: seconds(0.4) },
    });
    const mirror = {
      items: {
        a: item('a', 'a1', 'g'),
        v: item('v', 'v1', 'g'),
        other: item('other', 'v2', null),
      },
    } as unknown as ProjectMirror;
    const [passage] = cutPassages(
      [clip('a', 10, 0.4, 0), clip('b', 10.4, 1, 0.9)],
      () => TRANSCRIPT,
      WORDS,
    );
    if (passage === undefined) throw new Error('a passage');
    expect(restoreTrims(passage, mirror)).toEqual([
      { itemId: 'a', edge: 'end', to: seconds(10.9) },
      { itemId: 'v', edge: 'end', to: seconds(10.9) },
    ]);
    expect(restoreTrims({ ...passage, leftItemId: 'gone' }, mirror)).toBeNull();
  });
});

describe('silence options', () => {
  it('listens to unmuted, unlocked sound tracks holding media by default', () => {
    const track = (id: string, overrides: Partial<TrackEntity>): TrackEntity =>
      ({
        id,
        name: id.toUpperCase(),
        kind: 'audio',
        muted: false,
        locked: false,
        itemIds: ['x'],
        ...overrides,
      }) as TrackEntity;
    const mirror = {
      sequences: { s: { trackIds: ['v1', 'a1', 'a2', 'a3'] } },
      tracks: {
        v1: track('v1', { kind: 'video' }),
        a1: track('a1', {}),
        a2: track('a2', { muted: true }),
        a3: track('a3', { itemIds: [] }),
      },
      items: { x: { kind: { kind: 'mediaClip' } } },
    } as unknown as ProjectMirror;
    const tracks = soundTracksOf(mirror, 's');
    expect(tracks.map((entry) => [entry.id, entry.byDefault])).toEqual([
      ['a1', true],
      ['a2', false],
      ['a3', false],
    ]);
    expect(listenedTracks(tracks, null)).toBeNull();
    expect(listenedTracks(tracks, new Set(['a1']))).toBeNull();
    expect(listenedTracks(tracks, new Set(['a1', 'a2']))).toEqual(['a1', 'a2']);
  });

  it('turns in and out points into a range', () => {
    expect(inOutRange(null, seconds(5), seconds(10))).toBeNull();
    expect(inOutRange(seconds(2), null, seconds(10))).toEqual({
      start: seconds(2),
      duration: seconds(8),
    });
    expect(inOutRange(seconds(2), seconds(1), seconds(10))).toBeNull();
  });
});

describe('filler lists and formats', () => {
  it('parses comma-separated words and keeps French and English first', () => {
    expect(parseWords(' euh, euh ,\nyou   know,, ')).toEqual(['euh', 'you know']);
    expect(editedLanguages([{ language: 'de', words: ['äh'] }])).toEqual(['fr', 'en', 'de']);
    expect(withTexts(['fr', 'en'], { fr: 'euh, ben', en: ' ' })).toEqual([
      { language: 'fr', words: ['euh', 'ben'] },
    ]);
  });

  it('writes seconds the French way and shortens long passages', () => {
    expect(cutSeconds(seconds(2.44))).toBe('2,4');
    expect(cutSeconds(seconds(0))).toBe('0,0');
    const words = Array.from({ length: 14 }, (_, index) => ({ text: `m${String(index)}` }));
    expect(passageExcerpt(words, 3)).toBe('m0 m1 m2 …');
  });
});
