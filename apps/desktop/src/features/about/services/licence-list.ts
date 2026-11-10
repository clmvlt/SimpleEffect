import type { NoticeEntry } from '../../../shared/desktop';
import { searchable } from '../../../shared/text';

/** A licence notice with the stable key the list uses. */
export interface ListedNotice extends NoticeEntry {
  /** `source:name@version`, unique in the notices. */
  readonly key: string;
}

/** The notices with their keys, in the generated order (components, npm, Cargo). */
export function listedNotices(entries: readonly NoticeEntry[]): ListedNotice[] {
  return entries.map((entry) => ({
    ...entry,
    key: `${entry.source}:${entry.name}@${entry.version}`,
  }));
}

/**
 * The notices matching `query` (accent- and case-insensitive, every word in the name or the
 * licence).
 */
export function filterNotices(notices: readonly ListedNotice[], query: string): ListedNotice[] {
  const words = searchable(query)
    .split(/\s+/)
    .filter((word) => word !== '');
  if (words.length === 0) return [...notices];
  return notices.filter((notice) => {
    const text = searchable(`${notice.name} ${notice.license}`);
    return words.every((word) => text.includes(word));
  });
}

/** How many notices each source holds. */
export function sourceCounts(
  notices: readonly NoticeEntry[],
): Record<NoticeEntry['source'], number> {
  const counts = { component: 0, npm: 0, cargo: 0 };
  for (const notice of notices) counts[notice.source] += 1;
  return counts;
}
