import type { ExportInfo } from '@se/protocol';

/** The prefix of system notification tags naming an export (`export:<id>`). */
export const EXPORT_NOTIFICATION_PREFIX = 'export:';

/** How an export that just ended is announced. */
export type ExportNotice =
  | {
      readonly tone: 'success';
      readonly export: ExportInfo;
      readonly sizeBytes: number;
      /** What ffprobe found wrong in the file, if anything. */
      readonly warning: string | null;
    }
  | { readonly tone: 'error'; readonly export: ExportInfo; readonly message: string };

/**
 * The announcement of `next` when it just completed or failed (it was not before); `null`
 * otherwise. Cancelled exports are not announced: the user asked for it.
 */
export function exportNotice(
  previous: ExportInfo | undefined,
  next: ExportInfo,
): ExportNotice | null {
  if (previous?.state.kind === next.state.kind) return null;
  if (next.state.kind === 'completed') {
    return {
      tone: 'success',
      export: next,
      sizeBytes: next.state.sizeBytes,
      warning: next.state.check?.warning ?? null,
    };
  }
  if (next.state.kind === 'failed') {
    return { tone: 'error', export: next, message: next.state.message };
  }
  return null;
}

/** The notification tag of an export. */
export function exportNotificationTag(info: ExportInfo): string {
  return `${EXPORT_NOTIFICATION_PREFIX}${info.id}`;
}

/** The export a notification tag names, `null` for other tags. */
export function exportIdOfTag(tag: string): string | null {
  return tag.startsWith(EXPORT_NOTIFICATION_PREFIX)
    ? tag.slice(EXPORT_NOTIFICATION_PREFIX.length)
    : null;
}

/** The name of a file path (after its last separator, `\` or `/`), for messages. */
export function fileNameOf(path: string): string {
  return path.split(/[\\/]/).at(-1) ?? path;
}

/** How many media, fonts and templates a finished export could not draw. */
export function renderIssueCount(issues: {
  readonly missingAssets: readonly unknown[];
  readonly missingFonts: readonly unknown[];
  readonly failedTemplates: readonly unknown[];
}): number {
  return issues.missingAssets.length + issues.missingFonts.length + issues.failedTemplates.length;
}
