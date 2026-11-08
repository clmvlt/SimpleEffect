import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { findTimePhrase, type TimePhraseContext } from './time-phrases';

const SECOND = 705_600_000;
const seconds = (value: number) => Math.round(value * SECOND) as Flicks;

const CONTEXT: TimePhraseContext = {
  frameRate: { numerator: 25, denominator: 1 },
  sequenceEnd: seconds(180),
};

/** [prompt, expected start and end in seconds, expected words] */
const FOUND: readonly [string, number, number, string][] = [
  ['de 51s à 54s mets un titre', 51, 54, 'de 51s à 54s'],
  ['De 51 s à 54 s affiche « Abonne-toi »', 51, 54, 'De 51 s à 54 s'],
  ['de 51,6 s à 54 s', 51.6, 54, 'de 51,6 s à 54 s'],
  ['de 51 à 54 secondes', 51, 54, 'de 51 à 54 secondes'],
  ['de 51s a 54s', 51, 54, 'de 51s a 54s'],
  ['entre 1:02 et 1:10, coupe les silences', 62, 70, 'entre 1:02 et 1:10'],
  ['entre la 5e… non : entre 10 et 20 secondes', 10, 20, 'entre 10 et 20 secondes'],
  ['00:51-00:54 un zoom', 51, 54, '00:51-00:54'],
  ['00:51 → 00:54', 51, 54, '00:51 → 00:54'],
  ['51s-54s', 51, 54, '51s-54s'],
  ['ajoute un titre 51-54s', 51, 54, '51-54s'],
  ['de 1 min 05 à 1 min 20', 65, 80, 'de 1 min 05 à 1 min 20'],
  ['de 1m30 à 2m', 90, 120, 'de 1m30 à 2m'],
  ['depuis 10 s jusqu’à 12 s', 10, 12, 'depuis 10 s jusqu’à 12 s'],
  ['de 00:00:51:12 à 00:00:54:00', 51.48, 54, 'de 00:00:51:12 à 00:00:54:00'],
  ['de 0:01:02 à 0:01:10', 62, 70, 'de 0:01:02 à 0:01:10'],
  ['à partir de 2 min, accélère', 120, 180, 'à partir de 2 min'],
  ['dès 2 min 30 baisse la musique', 150, 180, 'dès 2 min 30'],
  ['jusqu’à 30 s', 0, 30, 'jusqu’à 30 s'],
  ['les 10 premières secondes', 0, 10, 'les 10 premières secondes'],
  ['la première minute', 0, 60, 'la première minute'],
  ['les 15 dernières secondes', 165, 180, 'les 15 dernières secondes'],
  ['les 2 dernières minutes', 60, 180, 'les 2 dernières minutes'],
  // The first range of the prompt wins.
  ['de 10s à 12s puis de 20s à 22s', 10, 12, 'de 10s à 12s'],
  // Bounds snap to the nearest frame (25 fps: 40 ms).
  ['de 1.01s à 2s', 1, 2, 'de 1.01s à 2s'],
];

/** Prompts with no usable range: ambiguous, invalid, reversed or empty. */
const NOT_FOUND: readonly string[] = [
  'mets un titre',
  'ajoute de 1 à 2 titres',
  'entre 2020 et 2024',
  'de 54s à 51s',
  'de 51s à 51s',
  'à 51s mets un titre',
  'de 1:75 à 2:00',
  'à partir de 3 min',
  'les premières secondes',
  'la 1 première minute',
  'garde les 10 premiers plans',
  'de 51 à 54 mets un titre',
  'de 50 à 1:10',
  'V1-V2',
];

describe('findTimePhrase', () => {
  it.each(FOUND)('reads « %s »', (prompt, start, end, text) => {
    const found = findTimePhrase(prompt, CONTEXT);
    expect(found).not.toBeNull();
    expect(found?.start).toBe(seconds(start));
    expect(found?.end).toBe(seconds(end));
    expect(found?.text).toBe(text);
    expect(prompt.slice(found?.index, (found?.index ?? 0) + text.length)).toBe(text);
  });

  it.each(NOT_FOUND)('finds nothing in « %s »', (prompt) => {
    expect(findTimePhrase(prompt, CONTEXT)).toBeNull();
  });

  it('needs the frame rate for SMPTE timecodes and reads them at the sequence rate', () => {
    const at30 = { ...CONTEXT, frameRate: { numerator: 30, denominator: 1 } };
    expect(findTimePhrase('de 00:00:01:15 à 00:00:02:00', at30)?.start).toBe(seconds(1.5));
  });
});
