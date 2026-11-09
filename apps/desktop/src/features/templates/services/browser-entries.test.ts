import type { LibraryTemplate, MotionTemplate } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { badgeTemplate } from '../../../shared/project-store/testing/template-fixture';
import {
  browserEntries,
  captionStylePatchOf,
  filterEntries,
  parseTags,
  previewTemplate,
  updateFor,
} from './browser-entries';
import { libraryEntry, libraryStyle } from '../testing/library-fixture';

function linked(version: number): MotionTemplate {
  return { ...badgeTemplate(), library: { templateId: 'entry-1', version } };
}

describe('template browser entries', () => {
  it('offers the library’s newer version as an update, never an older one', () => {
    const entry = libraryEntry({ version: 3 });
    expect(updateFor(linked(2), [entry])).toBe(entry);
    expect(updateFor(linked(3), [entry])).toBeNull();
    expect(updateFor(badgeTemplate(), [entry])).toBeNull();
  });

  it('lists project templates then the library, each by name, with what they need', () => {
    const entries = browserEntries(
      [linked(1)],
      [libraryEntry({ version: 2 }), libraryStyle()],
      new Set(['badge']),
    );
    expect(entries.map((entry) => entry.key)).toEqual([
      'project:badge',
      'library:entry-1',
      'library:style-1',
    ]);
    const [project, saved, style] = entries;
    expect(project?.kind === 'project' && project.used).toBe(true);
    expect(project?.kind === 'project' && project.update?.version).toBe(2);
    expect(saved?.kind === 'library' && saved.template?.id).toBe('library:entry-1');
    expect(style?.category).toBe('captions');
  });

  it('filters by source, category and words (accents and case ignored)', () => {
    const entries = browserEntries(
      [linked(1)],
      [libraryEntry({ version: 1 }), libraryStyle()],
      new Set(),
    );
    const keys = (filter: Parameters<typeof filterEntries>[1]) =>
      filterEntries(entries, filter).map((entry) => entry.key);
    // The library copy already in the project shows once, as the project card.
    expect(keys({ source: 'all', category: null, search: '' })).toEqual([
      'project:badge',
      'library:style-1',
    ]);
    expect(keys({ source: 'library', category: null, search: '' })).toEqual([
      'library:entry-1',
      'library:style-1',
    ]);
    expect(keys({ source: 'library', category: 'captions', search: '' })).toEqual([
      'library:style-1',
    ]);
    expect(keys({ source: 'all', category: null, search: 'INTERVIEW' })).toEqual([]);
    expect(keys({ source: 'library', category: null, search: 'intérview' })).toEqual([
      'library:entry-1',
    ]);
  });

  it('reads tags and caption looks', () => {
    expect(parseTags(' shorts, ,intro ')).toEqual(['shorts', 'intro']);
    const style = libraryStyle();
    const patch = captionStylePatchOf(
      (style.content as Extract<LibraryTemplate['content'], { kind: 'captionStyle' }>).style,
    );
    expect(patch.preset).toEqual({ value: 'box' });
    expect(patch.maxLines).toBe(2);
    expect(previewTemplate(style)).toBeNull();
  });
});
