/**
 * The history of sent prompts, browsed with ↑/↓ in the composer like a shell: ↑ in an empty field
 * (or on an entry being browsed) shows the previous prompt, ↓ the next one and finally the draft.
 */

/** Prompts kept (newest last). */
export const MAX_HISTORY = 50;

/** Where browsing stands. */
export interface HistoryBrowsing {
  /** Index of the entry shown; `null` when not browsing. */
  readonly cursor: number | null;
  /** What the field held before browsing started. */
  readonly draft: string;
}

/** Not browsing. */
export const NOT_BROWSING: HistoryBrowsing = { cursor: null, draft: '' };

/** `entries` with `prompt` added last (an earlier copy moves), at most {@link MAX_HISTORY}. */
export function rememberPrompt(entries: readonly string[], prompt: string): string[] {
  const text = prompt.trim();
  if (text === '') return [...entries];
  return [...entries.filter((entry) => entry !== text), text].slice(-MAX_HISTORY);
}

/** The field's text and browsing state after one ↑ or ↓; `null` when the key does nothing. */
export function browseHistory(
  entries: readonly string[],
  browsing: HistoryBrowsing,
  value: string,
  direction: 'previous' | 'next',
): { readonly value: string; readonly browsing: HistoryBrowsing } | null {
  const { cursor } = browsing;
  const onEntry = cursor !== null && entries[cursor] === value;
  if (direction === 'previous') {
    if (onEntry) {
      if (cursor === 0) return null;
      return { value: entries[cursor - 1] ?? value, browsing: { ...browsing, cursor: cursor - 1 } };
    }
    if (value !== '' || entries.length === 0) return null;
    const last = entries.length - 1;
    return { value: entries[last] ?? '', browsing: { cursor: last, draft: value } };
  }
  if (!onEntry) return null;
  if (cursor === entries.length - 1) return { value: browsing.draft, browsing: NOT_BROWSING };
  return { value: entries[cursor + 1] ?? value, browsing: { ...browsing, cursor: cursor + 1 } };
}
