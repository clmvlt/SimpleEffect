// Detection of raw colour literals in UI source code (golden rule 9: design tokens only).
// Colours are defined once, in `tokens/` folders; everything else must use token utilities
// (`bg-panel`, `text-text-muted`…) or `var(--color-…)`.
import { readdir, readFile } from 'node:fs/promises';
import { extname, join, relative, sep } from 'node:path';

/** Folders scanned by `pnpm check`, relative to the repository root. */
export const SCANNED_DIRECTORIES = ['packages/ui/src', 'apps/desktop/src'];

/** Folder names never scanned: `tokens` holds the palette itself; the others are not sources. */
const SKIPPED_DIRECTORY_NAMES = new Set(['tokens', 'node_modules', '.tsbuild', 'storybook-static']);

const SCANNED_EXTENSIONS = new Set(['.ts', '.tsx', '.js', '.jsx', '.mjs', '.css', '.html', '.mdx']);

/**
 * Hex colours (`#fff`, `#ffffff80`…), colour functions (`rgb(`, `oklch(`…) and `color(<space> …)`.
 * A hex match must end at a word boundary, so `#fade-in` or `#header` do not match.
 */
const RAW_COLOR_PATTERN =
  /#(?:[0-9a-fA-F]{8}|[0-9a-fA-F]{6}|[0-9a-fA-F]{3,4})\b|\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(|\bcolor\(\s*(?:srgb|srgb-linear|display-p3|a98-rgb|prophoto-rgb|rec2020|xyz|xyz-d50|xyz-d65)\b/g;

/**
 * @typedef {object} RawColor
 * @property {number} line 1-based line number.
 * @property {number} column 1-based column number.
 * @property {string} literal The matched text.
 */

/**
 * Finds the raw colour literals in a source text.
 *
 * @param {string} source
 * @returns {RawColor[]}
 */
export function findRawColors(source) {
  return source.split(/\r?\n/).flatMap((text, index) =>
    Array.from(text.matchAll(RAW_COLOR_PATTERN), (match) => ({
      line: index + 1,
      column: match.index + 1,
      literal: match[0],
    })),
  );
}

/**
 * Lists the source files under `directory`, skipping token and build folders.
 *
 * @param {string} directory Absolute path.
 * @returns {Promise<string[]>} Absolute paths.
 */
async function listSourceFiles(directory) {
  const entries = await readdir(directory, { withFileTypes: true }).catch(() => []);
  const nested = await Promise.all(
    entries.map(async (entry) => {
      const path = join(directory, entry.name);
      if (entry.isDirectory()) {
        return SKIPPED_DIRECTORY_NAMES.has(entry.name) ? [] : listSourceFiles(path);
      }
      return SCANNED_EXTENSIONS.has(extname(entry.name)) ? [path] : [];
    }),
  );
  return nested.flat();
}

/**
 * Scans `directories` (relative to `root`) and returns every raw colour with its file, as a
 * forward-slash path relative to `root`.
 *
 * @param {string} root Absolute path of the repository (or a test fixture).
 * @param {readonly string[]} [directories]
 * @returns {Promise<Array<RawColor & { file: string }>>}
 */
export async function scanForRawColors(root, directories = SCANNED_DIRECTORIES) {
  const files = (
    await Promise.all(directories.map((dir) => listSourceFiles(join(root, dir))))
  ).flat();
  const findings = await Promise.all(
    files.sort().map(async (path) => {
      const file = relative(root, path).split(sep).join('/');
      return findRawColors(await readFile(path, 'utf8')).map((found) => ({ file, ...found }));
    }),
  );
  return findings.flat();
}
