// Pure helpers of `pnpm fetch-binaries`: arguments, manifest validation, platform keys, plans and
// up-to-date checks. The IO (download, hashing, extraction) lives in `fetch-binaries.mjs`.

/** Platform keys of `resources/bin/<platform>/` (docs/architecture.md §2). */
export const PLATFORMS = ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64'];

/** Archive kinds the script can extract. */
export const ARCHIVES = ['zip', 'tar.xz', 'tar.gz', 'none'];

/** Name of the file recording what was installed in a platform folder. */
export const STAMP_FILE = '.fetched.json';

/**
 * The platform key of a Node `process.platform` / `process.arch` pair, or `null` if unsupported.
 *
 * @param {string} platform
 * @param {string} arch
 * @returns {string | null}
 */
export function platformKey(platform, arch) {
  const key = `${platform}-${arch}`;
  return PLATFORMS.includes(key) ? key : null;
}

/**
 * Parses `--yes`, `--platform <key>` (repeatable) and `--all`.
 *
 * @param {readonly string[]} argv
 * @returns {{ yes: boolean, platforms: string[] | null, all: boolean }}
 */
export function parseArgs(argv) {
  const options = { yes: false, platforms: /** @type {string[] | null} */ (null), all: false };
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index];
    if (argument === '--yes' || argument === '-y') {
      options.yes = true;
    } else if (argument === '--all') {
      options.all = true;
    } else if (argument === '--platform') {
      const value = argv[index + 1];
      if (value === undefined || !PLATFORMS.includes(value)) {
        throw new Error(`--platform expects one of ${PLATFORMS.join(', ')}`);
      }
      options.platforms = [...(options.platforms ?? []), value];
      index += 1;
    } else {
      throw new Error(`unknown argument "${argument}" (use --yes, --platform <key>, --all)`);
    }
  }
  return options;
}

/**
 * Checks the manifest's shape and returns it; throws with every problem found.
 *
 * @param {unknown} manifest
 * @returns {Manifest}
 */
export function validateManifest(manifest) {
  const problems = [];
  const platforms = /** @type {Record<string, any>} */ (manifest)?.platforms;
  if (typeof platforms !== 'object' || platforms === null) {
    throw new Error('the manifest has no "platforms" object');
  }
  for (const key of PLATFORMS) {
    const entry = platforms[key];
    if (!entry || !Array.isArray(entry.downloads) || entry.downloads.length === 0) {
      problems.push(`${key}: no downloads`);
      continue;
    }
    if (typeof entry.license !== 'string' || entry.license.length === 0) {
      problems.push(`${key}: no license`);
    }
    entry.downloads.forEach((download, index) => {
      problems.push(...downloadProblems(download).map((problem) => `${key}[${index}]: ${problem}`));
    });
  }
  if (problems.length > 0) {
    throw new Error(`invalid binaries manifest:\n  ${problems.join('\n  ')}`);
  }
  return /** @type {Manifest} */ (manifest);
}

/** @param {any} download */
function downloadProblems(download) {
  const problems = [];
  if (typeof download?.url !== 'string' || !download.url.startsWith('https://')) {
    problems.push('url must be https');
  }
  if (typeof download?.sha256 !== 'string' || !/^[0-9a-f]{64}$/.test(download.sha256)) {
    problems.push('sha256 must be 64 lowercase hex digits');
  }
  if (!Number.isInteger(download?.size) || download.size <= 0) {
    problems.push('size must be a positive integer');
  }
  if (!ARCHIVES.includes(download?.archive)) {
    problems.push(`archive must be one of ${ARCHIVES.join(', ')}`);
  }
  const files = download?.files;
  if (typeof files !== 'object' || files === null || Object.keys(files).length === 0) {
    problems.push('files must map installed names to archive paths');
  } else if (Object.keys(files).some((name) => name.includes('/') || name.includes('\\'))) {
    problems.push('installed names must be plain file names');
  }
  return problems;
}

/**
 * Human-readable lines describing what will be downloaded for `key`.
 *
 * @param {Manifest} manifest
 * @param {string} key
 * @returns {string[]}
 */
export function describePlan(manifest, key) {
  const entry = manifest.platforms[key];
  const lines = [`${key} — FFmpeg ${manifest.ffmpegVersion}, ${entry.license}`];
  for (const download of entry.downloads) {
    lines.push(`  ${formatSize(download.size)}  ${download.url}`);
  }
  const total = entry.downloads.reduce((sum, download) => sum + download.size, 0);
  lines.push(`  total ${formatSize(total)}`);
  return lines;
}

/**
 * Whether a platform folder's stamp records exactly these downloads (nothing to fetch).
 *
 * @param {unknown} stamp
 * @param {PlatformEntry} entry
 */
export function isUpToDate(stamp, entry) {
  const recorded = /** @type {{ sha256?: unknown }} */ (stamp)?.sha256;
  if (!Array.isArray(recorded)) {
    return false;
  }
  const wanted = entry.downloads.map((download) => download.sha256);
  return (
    recorded.length === wanted.length && wanted.every((hash, index) => recorded[index] === hash)
  );
}

/** The stamp written after installing `entry`. @param {PlatformEntry} entry */
export function stampFor(entry) {
  return { sha256: entry.downloads.map((download) => download.sha256) };
}

/** `114.8 MB`. @param {number} bytes */
export function formatSize(bytes) {
  return `${(bytes / 1_000_000).toFixed(1)} MB`;
}

/**
 * @typedef {{ url: string, sha256: string, size: number, archive: 'zip' | 'tar.xz' | 'tar.gz' | 'none', files: Record<string, string> }} Download
 * @typedef {{ license: string, downloads: Download[] }} PlatformEntry
 * @typedef {{ ffmpegVersion: string, platforms: Record<string, PlatformEntry> }} Manifest
 */
