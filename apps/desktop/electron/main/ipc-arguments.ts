import { isAbsolute } from 'node:path';

import type { ChooseFilesOptions, ChooseFolderOptions } from '../shared/bridge-api';

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

/** A path to reveal in the file manager: an absolute path string. */
export function parseRevealPath(value: unknown): string {
  if (typeof value !== 'string' || !isAbsolute(value)) {
    throw new TypeError('showInFolder expects an absolute path');
  }
  return value;
}
