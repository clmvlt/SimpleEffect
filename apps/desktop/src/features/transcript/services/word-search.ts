import type { SequenceWord } from '../../../shared/transcript';

/**
 * Searching the words of a transcript: accents, case and punctuation ignored; every searched word
 * must match consecutive transcript words, the last one as a prefix (« abonn » finds
 * « Abonnez-vous »).
 */

/** Lowercase, without diacritics, punctuation and apostrophes turned into spaces. */
export function foldText(text: string): string {
  return text
    .normalize('NFD')
    .replace(/\p{M}/gu, '')
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, ' ')
    .trim();
}

/** A match: the indices of its first and last word. */
export interface WordMatch {
  readonly first: number;
  readonly last: number;
}

/** The pieces of each word once folded (`l'homme,` → `l`, `homme`). */
function foldedPieces(words: readonly Pick<SequenceWord, 'text'>[]): string[][] {
  return words.map((word) => foldText(word.text).split(' ').filter(Boolean));
}

/** Every match of `query` in `words`, in order (none for an empty query). */
export function findMatches(
  words: readonly Pick<SequenceWord, 'text'>[],
  query: string,
): WordMatch[] {
  const terms = foldText(query).split(' ').filter(Boolean);
  if (terms.length === 0) return [];
  const pieces = foldedPieces(words);
  const flat: { piece: string; word: number }[] = [];
  pieces.forEach((parts, word) => {
    for (const piece of parts) flat.push({ piece, word });
  });
  const matches: WordMatch[] = [];
  for (let start = 0; start + terms.length <= flat.length; start += 1) {
    const ok = terms.every((term, offset) => {
      const candidate = flat[start + offset]?.piece ?? '';
      return offset === terms.length - 1 ? candidate.startsWith(term) : candidate === term;
    });
    const first = flat[start]?.word;
    const last = flat[start + terms.length - 1]?.word;
    if (ok && first !== undefined && last !== undefined) {
      const previous = matches.at(-1);
      if (previous?.first !== first) matches.push({ first, last });
    }
  }
  return matches;
}

/** The index of the match after (`step` 1) or before (`step` −1) `current`, wrapping around. */
export function stepMatch(count: number, current: number | null, step: 1 | -1): number | null {
  if (count === 0) return null;
  if (current === null) return step === 1 ? 0 : count - 1;
  return (current + step + count) % count;
}
