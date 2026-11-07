import { isAbsolute } from 'node:path';

import type { ChooseFolderOptions } from '../shared/bridge-api';

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

/** A path to reveal in the file manager: an absolute path string. */
export function parseRevealPath(value: unknown): string {
  if (typeof value !== 'string' || !isAbsolute(value)) {
    throw new TypeError('showInFolder expects an absolute path');
  }
  return value;
}
