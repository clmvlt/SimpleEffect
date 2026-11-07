import type { ErrorCode, ImportReport } from '@se/protocol';

import { fileName } from './asset-list';

/** What to tell the user after an import. */
export interface ImportSummary {
  readonly imported: number;
  readonly duplicates: number;
  /** File names that failed, with their error code. */
  readonly failures: readonly { name: string; code: ErrorCode }[];
}

/** Counts and failures of an import report. */
export function summarizeImport(report: ImportReport): ImportSummary {
  return {
    imported: report.imported.length,
    duplicates: report.duplicates.length,
    failures: report.failures.map((failure) => ({
      name: fileName(failure.path),
      code: failure.code,
    })),
  };
}

/** Absolute paths of files dropped from the OS (files without a path, e.g. from a web page, are skipped). */
export function droppedPaths(
  files: Iterable<File>,
  pathOf: (file: File) => string,
): readonly string[] {
  return [...files].map(pathOf).filter((path) => path !== '');
}
