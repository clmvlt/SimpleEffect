/**
 * Pure helpers of the caption list's text editing: where the caret splits a caption, and what a
 * keystroke asks for.
 */

/**
 * The word of `text` the caret at `cursor` (a UTF-16 offset) makes start a new caption: the
 * first word beginning at or after the caret (a caret inside a word splits before it). `null`
 * when that leaves no word on one side.
 */
export function splitWordIndex(text: string, cursor: number): number | null {
  const words = [...text.matchAll(/\S+/g)];
  const index = words.findIndex((match) => match.index + match[0].length > cursor);
  if (index <= 0 || index >= words.length) return null;
  return index;
}

/** What a key pressed in a caption's text field asks for. */
export type CaptionKeyAction = 'split' | 'mergeWithPrevious' | 'commit' | 'cancel' | null;

/** The action of a key: Entrée splits at the caret (Ctrl/⌘+Entrée commits), Retour arrière at the
 * very start merges with the previous caption, Échap cancels. */
export function captionKeyAction(
  key: {
    readonly key: string;
    readonly ctrlKey: boolean;
    readonly metaKey: boolean;
    readonly shiftKey: boolean;
  },
  caret: { readonly start: number; readonly end: number },
): CaptionKeyAction {
  if (key.key === 'Escape') return 'cancel';
  if (key.key === 'Enter') return key.ctrlKey || key.metaKey ? 'commit' : 'split';
  if (key.key === 'Backspace' && caret.start === 0 && caret.end === 0) {
    return 'mergeWithPrevious';
  }
  return null;
}

/** Seconds for the shift field: `-0,5` or `1.25` (French or English decimals), `null` otherwise. */
export function parseShiftSeconds(text: string): number | null {
  const normalized = text.trim().replace(',', '.').replace(/\s*s$/i, '');
  if (!/^[+-]?\d+(\.\d+)?$/.test(normalized)) return null;
  const value = Number(normalized);
  return Number.isFinite(value) && value !== 0 ? value : null;
}
