import { isAbsolute } from 'node:path';

import type {
  ChooseFileOptions,
  ChooseFilesOptions,
  ChooseSaveFileOptions,
  ChooseFolderOptions,
  SystemNotification,
} from '../shared/bridge-api';

/** Longest notification title or text accepted (the OS shortens them further). */
const MAX_NOTIFICATION_TEXT = 500;

/**
 * Validation of IPC arguments coming from the renderer. The renderer is sandboxed but still
 * untrusted input for the main process: anything malformed is refused with an error.
 */

/** Options of the folder picker, checked. */
export function parseChooseFolderOptions(value: unknown): ChooseFolderOptions {
  if (typeof value !== 'object' || value === null) {
    throw new TypeError('chooseFolder expects an options object');
  }
  const { title, buttonLabel, defaultPath } = value as Record<string, unknown>;
  if (typeof title !== 'string' || typeof buttonLabel !== 'string') {
    throw new TypeError('chooseFolder expects string `title` and `buttonLabel`');
  }
  if (defaultPath === undefined) {
    return { title, buttonLabel };
  }
  if (typeof defaultPath !== 'string') {
    throw new TypeError('chooseFolder expects `defaultPath` to be a string when present');
  }
  return { title, buttonLabel, defaultPath };
}

/** Options of the file picker, checked. */
export function parseChooseFilesOptions(value: unknown): ChooseFilesOptions {
  if (typeof value !== 'object' || value === null) {
    throw new TypeError('chooseFiles expects an options object');
  }
  const { title, buttonLabel, filterName, extensions, allFilesName } = value as Record<
    string,
    unknown
  >;
  const strings = [title, buttonLabel, filterName, allFilesName];
  if (!strings.every((text) => typeof text === 'string')) {
    throw new TypeError(
      'chooseFiles expects string `title`, `buttonLabel`, `filterName` and `allFilesName`',
    );
  }
  const validExtensions =
    Array.isArray(extensions) &&
    extensions.every(
      (extension) => typeof extension === 'string' && /^[a-z0-9]+$/i.test(extension),
    );
  if (!validExtensions) {
    throw new TypeError('chooseFiles expects `extensions` to be a list of letters and digits');
  }
  return {
    title: title as string,
    buttonLabel: buttonLabel as string,
    filterName: filterName as string,
    allFilesName: allFilesName as string,
    extensions: extensions as string[],
  };
}

/** Options of the save dialog, checked. */
export function parseChooseSaveFileOptions(value: unknown): ChooseSaveFileOptions {
  if (typeof value !== 'object' || value === null) {
    throw new TypeError('chooseSaveFile expects an options object');
  }
  const { title, buttonLabel, defaultName, filterName, extensions } = value as Record<
    string,
    unknown
  >;
  if (![title, buttonLabel, defaultName, filterName].every((text) => typeof text === 'string')) {
    throw new TypeError(
      'chooseSaveFile expects string `title`, `buttonLabel`, `defaultName` and `filterName`',
    );
  }
  if (/[\\/]/.test(defaultName as string)) {
    throw new TypeError('chooseSaveFile expects `defaultName` to be a file name, not a path');
  }
  const validExtensions =
    Array.isArray(extensions) &&
    extensions.every(
      (extension) => typeof extension === 'string' && /^[a-z0-9]+$/i.test(extension),
    );
  if (!validExtensions) {
    throw new TypeError('chooseSaveFile expects `extensions` to be a list of letters and digits');
  }
  return {
    title: title as string,
    buttonLabel: buttonLabel as string,
    defaultName: defaultName as string,
    filterName: filterName as string,
    extensions: extensions as string[],
  };
}

/** A path to reveal in the file manager: an absolute path string. */
export function parseRevealPath(value: unknown): string {
  if (typeof value !== 'string' || !isAbsolute(value)) {
    throw new TypeError('showInFolder expects an absolute path');
  }
  return value;
}

/** Options of the single-file picker, checked. */
export function parseChooseFileOptions(value: unknown): ChooseFileOptions {
  if (typeof value !== 'object' || value === null) {
    throw new TypeError('chooseFile expects an options object');
  }
  const { title, buttonLabel } = value as Record<string, unknown>;
  if (typeof title !== 'string' || typeof buttonLabel !== 'string') {
    throw new TypeError('chooseFile expects string `title` and `buttonLabel`');
  }
  return { title, buttonLabel };
}

/** A page to open in the browser: an `https:` URL without credentials. */
export function parseExternalUrl(value: unknown): string {
  if (typeof value !== 'string' || !URL.canParse(value)) {
    throw new TypeError('openExternal expects a URL');
  }
  const url = new URL(value);
  if (url.protocol !== 'https:' || url.username !== '' || url.password !== '') {
    throw new TypeError('openExternal only opens https pages');
  }
  return url.href;
}

/** A system notification, checked: short strings only. */
export function parseNotification(value: unknown): SystemNotification {
  if (typeof value !== 'object' || value === null) {
    throw new TypeError('showNotification expects a notification object');
  }
  const { title, body, tag } = value as Record<string, unknown>;
  const valid = [title, body, tag].every(
    (text) => typeof text === 'string' && text.length <= MAX_NOTIFICATION_TEXT,
  );
  if (!valid) {
    throw new TypeError(
      'showNotification expects string title, body and tag of at most ' +
        String(MAX_NOTIFICATION_TEXT) +
        ' characters',
    );
  }
  return { title: title as string, body: body as string, tag: tag as string };
}
