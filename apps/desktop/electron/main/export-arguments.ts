import { basename, isAbsolute } from 'node:path';

import type { ExportActivity, ResolveFilePathOptions } from '../shared/bridge-api';

/** Longest text of the export confirmation accepted. */
const MAX_ACTIVITY_TEXT = 500;
/** Longest file name accepted (file systems allow 255 bytes). */
const MAX_FILE_NAME = 255;

/**
 * Validation of the export IPC arguments (prompt 27). The renderer is sandboxed but still
 * untrusted input for the main process: anything malformed is refused with an error.
 */

/** A folder and a file name in it, checked: an absolute folder, a plain name. */
export function parseResolveFilePath(value: unknown): ResolveFilePathOptions {
  if (typeof value !== 'object' || value === null) {
    throw new TypeError('resolveFilePath expects an options object');
  }
  const { folder, fileName } = value as Record<string, unknown>;
  if (typeof folder !== 'string' || !isAbsolute(folder)) {
    throw new TypeError('resolveFilePath expects an absolute `folder`');
  }
  const plainName =
    typeof fileName === 'string' &&
    fileName.length > 0 &&
    fileName.length <= MAX_FILE_NAME &&
    !/[\\/]/.test(fileName) &&
    basename(fileName) === fileName &&
    fileName !== '.' &&
    fileName !== '..';
  if (!plainName) {
    throw new TypeError('resolveFilePath expects `fileName` to be a file name without folders');
  }
  return { folder, fileName };
}

/** The exports' activity (or `null` when none run), checked: short strings only. */
export function parseExportActivity(value: unknown): ExportActivity | null {
  if (value === null) return null;
  if (typeof value !== 'object') {
    throw new TypeError('setExportActivity expects an activity object or null');
  }
  const { title, message, quitLabel, stayLabel } = value as Record<string, unknown>;
  const texts = [title, message, quitLabel, stayLabel];
  if (!texts.every((text) => typeof text === 'string' && text.length <= MAX_ACTIVITY_TEXT)) {
    throw new TypeError(
      'setExportActivity expects string title, message, quitLabel and stayLabel of at most ' +
        String(MAX_ACTIVITY_TEXT) +
        ' characters',
    );
  }
  return {
    title: title as string,
    message: message as string,
    quitLabel: quitLabel as string,
    stayLabel: stayLabel as string,
  };
}
