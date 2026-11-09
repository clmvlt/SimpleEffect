import type {
  CaptionStyle,
  CaptionStylePatch,
  LibraryTemplate,
  MotionTemplate,
  TemplateCategory,
} from '@se/protocol';

import { searchable } from '../../../shared/text';

/** A category of the browser: the five template categories, and the library's caption styles. */
export type BrowserCategory = TemplateCategory | 'captions';

/** Every category, in the browser's order. */
export const BROWSER_CATEGORIES: readonly BrowserCategory[] = [
  'title',
  'lowerThird',
  'callToAction',
  'transition',
  'captions',
  'other',
];

/** Where the browser lists from. */
export type BrowserSource = 'all' | 'project' | 'library';

/** A template of the project. */
export interface ProjectEntry {
  readonly kind: 'project';
  readonly key: string;
  readonly template: MotionTemplate;
  readonly category: BrowserCategory;
  readonly name: string;
  readonly description: string;
  readonly tags: readonly string[];
  /** Whether an item uses it (it cannot be removed then). */
  readonly used: boolean;
  /** The newer library version offered as an update, if any. */
  readonly update: LibraryTemplate | null;
}

/** An entry of the template library. */
export interface LibraryEntry {
  readonly kind: 'library';
  readonly key: string;
  readonly entry: LibraryTemplate;
  /** Its motion template, shaped for previews (`null` for caption styles). */
  readonly template: MotionTemplate | null;
  /** Its caption look (`null` for motion templates). */
  readonly captionStyle: CaptionStyle | null;
  readonly category: BrowserCategory;
  readonly name: string;
  readonly description: string;
  readonly tags: readonly string[];
  /** Whether the project holds a copy of this very version. */
  readonly inProject: boolean;
}

/** One card of the browser. */
export type BrowserEntry = ProjectEntry | LibraryEntry;

/** What the browser shows. */
export interface BrowserFilter {
  readonly source: BrowserSource;
  readonly category: BrowserCategory | null;
  readonly search: string;
}

/** The library entry's motion template, shaped like a project template for previews. */
export function previewTemplate(entry: LibraryTemplate): MotionTemplate | null {
  const content = entry.content;
  if (content.kind !== 'motionTemplate') return null;
  return {
    id: `library:${entry.id}`,
    name: entry.name,
    description: entry.description,
    version: entry.version,
    source: content.source,
    params: content.params,
    defaultDuration: content.defaultDuration,
    origin: { kind: 'user' },
    category: content.category,
    library: { templateId: entry.id, version: entry.version },
  };
}

/** The library entry newer than `template`'s copy, if any (an update to propose). */
export function updateFor(
  template: MotionTemplate,
  library: readonly LibraryTemplate[],
): LibraryTemplate | null {
  const link = template.library;
  if (link === null) return null;
  const entry = library.find((candidate) => candidate.id === link.templateId);
  return entry !== undefined && entry.version > link.version ? entry : null;
}

/** Whether `templates` hold a copy of `entry`'s version. */
function hasCopy(templates: readonly MotionTemplate[], entry: LibraryTemplate): boolean {
  return templates.some(
    (template) =>
      template.library?.templateId === entry.id && template.library.version === entry.version,
  );
}

/** The cards of the project's templates and of the library, project first, each by name. */
export function browserEntries(
  templates: readonly MotionTemplate[],
  library: readonly LibraryTemplate[],
  usedTemplateIds: ReadonlySet<string>,
): readonly BrowserEntry[] {
  const byName = <T extends { readonly name: string }>(left: T, right: T) =>
    left.name.localeCompare(right.name, 'fr');
  const project: ProjectEntry[] = templates.map((template) => ({
    kind: 'project',
    key: `project:${template.id}`,
    template,
    category: template.category,
    name: template.name,
    description: template.description,
    tags: [],
    used: usedTemplateIds.has(template.id),
    update: updateFor(template, library),
  }));
  const saved: LibraryEntry[] = library.map((entry) => ({
    kind: 'library',
    key: `library:${entry.id}`,
    entry,
    template: previewTemplate(entry),
    captionStyle: entry.content.kind === 'captionStyle' ? entry.content.style : null,
    category: entry.content.kind === 'captionStyle' ? 'captions' : entry.content.category,
    name: entry.name,
    description: entry.description,
    tags: entry.tags,
    inProject: hasCopy(templates, entry),
  }));
  return [...project.sort(byName), ...saved.sort(byName)];
}

/**
 * The cards `filter` keeps: its source (« Tout » leaves out library entries the project already
 * holds in that version, shown once as project cards), its category, and the words searched in
 * names, descriptions and tags (accents and case ignored).
 */
export function filterEntries(
  entries: readonly BrowserEntry[],
  filter: BrowserFilter,
): readonly BrowserEntry[] {
  const words = searchable(filter.search).split(/\s+/u).filter(Boolean);
  return entries.filter((entry) => {
    if (filter.source === 'project' && entry.kind !== 'project') return false;
    if (filter.source === 'library' && entry.kind !== 'library') return false;
    if (filter.source === 'all' && entry.kind === 'library' && entry.inProject) return false;
    if (filter.category !== null && entry.category !== filter.category) return false;
    const haystack = searchable([entry.name, entry.description, ...entry.tags].join(' '));
    return words.every((word) => haystack.includes(word));
  });
}

/** Tags typed as text: separated by commas, trimmed, without empty ones. */
export function parseTags(text: string): readonly string[] {
  return text
    .split(',')
    .map((tag) => tag.trim())
    .filter((tag) => tag !== '');
}

/** A patch giving captions every value of `style` (a library caption look). */
export function captionStylePatchOf(style: CaptionStyle): CaptionStylePatch {
  return {
    preset: { value: style.preset },
    fontFamily: style.fontFamily,
    fontWeight: style.fontWeight,
    italic: style.italic,
    fontSize: style.fontSize,
    textCase: style.textCase,
    lineHeight: style.lineHeight,
    textColor: style.textColor,
    activeColor: style.activeColor,
    emphasisColor: style.emphasisColor,
    outline: style.outline,
    shadow: style.shadow,
    background: style.background,
    position: style.position,
    animation: style.animation,
    highlightSpoken: style.highlightSpoken,
    maxCharsPerLine: style.maxCharsPerLine,
    maxLines: style.maxLines,
  };
}
