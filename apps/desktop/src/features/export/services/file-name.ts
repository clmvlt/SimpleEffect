import type { ExportContainer } from '@se/protocol';

/** What a file name template can name. */
export interface FileNameValues {
  readonly project: string;
  readonly sequence: string;
  readonly preset: string;
  /** When the export starts (local date and time). */
  readonly date: Date;
  /** The picture size, absent for sound-only exports. */
  readonly width?: number;
  readonly height?: number;
}

/** The placeholders of a template (also listed in the dialog's hint). */
export const FILE_NAME_FIELDS = [
  'project',
  'sequence',
  'preset',
  'date',
  'time',
  'width',
  'height',
] as const;

/** Characters no file name may hold on Windows, macOS or Linux (the core refuses them too). */
const FORBIDDEN = /[<>:"/\\|?*\p{Cc}]/gu;
/** Longest name kept (without extension): well below every file system's limit. */
const MAX_NAME_CHARS = 180;

/** Longest template the core saves. */
const MAX_TEMPLATE_CHARS = 200;

/** Whether the core accepts `template` as the file name template it remembers. */
export function isValidTemplate(template: string): boolean {
  return (
    template.trim() !== '' &&
    // Counted in code points, as the core counts `char`s.
    Array.from(template).length <= MAX_TEMPLATE_CHARS &&
    !new RegExp(FORBIDDEN.source, 'u').test(template)
  );
}

const pad = (value: number) => String(value).padStart(2, '0');

function fieldValue(field: string, values: FileNameValues): string | null {
  const { date } = values;
  switch (field) {
    case 'project':
      return values.project;
    case 'sequence':
      return values.sequence;
    case 'preset':
      return values.preset;
    case 'date':
      return `${String(date.getFullYear())}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
    case 'time':
      return `${pad(date.getHours())}h${pad(date.getMinutes())}`;
    case 'width':
      return values.width === undefined ? '' : String(values.width);
    case 'height':
      return values.height === undefined ? '' : String(values.height);
    default:
      return null;
  }
}

/**
 * The file name (without extension) a template gives: placeholders replaced (unknown ones kept
 * as typed), characters file systems refuse replaced by `_`, spaces and dots trimmed at the ends
 * (Windows drops trailing ones); `export` when nothing is left.
 */
export function expandFileName(template: string, values: FileNameValues): string {
  const expanded = template.replace(
    /\{(\w+)\}/g,
    (whole, field: string) => fieldValue(field, values) ?? whole,
  );
  const safe = expanded
    .replace(FORBIDDEN, '_')
    .replace(/\s+/g, ' ')
    .slice(0, MAX_NAME_CHARS)
    .replace(/^[\s.]+|[\s.]+$/g, '');
  return safe === '' ? 'export' : safe;
}

/** The extension of a container's files. */
export function extensionOf(container: ExportContainer): string {
  return container;
}

/** `name` with the extension of `container`. */
export function fileNameWithExtension(name: string, container: ExportContainer): string {
  return `${name}.${extensionOf(container)}`;
}
