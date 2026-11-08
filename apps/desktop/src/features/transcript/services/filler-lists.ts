import type { FillerList } from '@se/protocol';

/** The languages whose list the editor always shows (others appear when they have one). */
export const EDITED_LANGUAGES = ['fr', 'en'] as const;

/** A list's words as the editor shows them: comma-separated. */
export function wordsText(list: FillerList | undefined): string {
  return list === undefined ? '' : list.words.join(', ');
}

/** Words typed comma- or line-separated, trimmed, empty and repeated ones dropped. */
export function parseWords(text: string): string[] {
  const words: string[] = [];
  for (const raw of text.split(/[,\n]/)) {
    const word = raw.trim().replace(/\s+/g, ' ');
    if (word !== '' && !words.includes(word)) words.push(word);
  }
  return words;
}

/** The languages to edit: the always shown ones, then the others of `lists`. */
export function editedLanguages(lists: readonly FillerList[]): string[] {
  const languages: string[] = [...EDITED_LANGUAGES];
  for (const list of lists) {
    if (!languages.includes(list.language)) languages.push(list.language);
  }
  return languages;
}

/** `lists` with the texts typed per language (empty lists dropped). */
export function withTexts(languages: readonly string[], texts: Readonly<Record<string, string>>) {
  return languages.flatMap((language): FillerList[] => {
    const words = parseWords(texts[language] ?? '');
    return words.length === 0 ? [] : [{ language, words }];
  });
}
