// Pure helpers of `pnpm fetch-binaries`: arguments, manifest validation, platform keys, plans,
// source builds and up-to-date checks. The IO (download, hashing, extraction, building) lives in
// `fetch-binaries.mjs`.

/** Platform keys of `resources/bin/<platform>/` (docs/architecture.md §2). */
export const PLATFORMS = ['win32-x64', 'darwin-arm64', 'darwin-x64', 'linux-x64'];

/** Archive kinds the script can extract. */
export const ARCHIVES = ['zip', 'tar.xz', 'tar.gz', 'none'];

/** Name of the file recording what was installed in a platform folder. */
export const STAMP_FILE = '.fetched.json';

/**
 * Build systems a source download may use: the program run for each step, and what to tell a
 * developer who does not have it.
 */
export const BUILD_SYSTEMS = {
  cmake: {
    program: 'cmake',
    hint:
      'install CMake (e.g. `brew install cmake`) and a C/C++ compiler ' +
      '(Xcode command line tools: `xcode-select --install`), then run the script again',
  },
};

/** Placeholders a build step may contain, replaced by `buildCommands`. */
export const BUILD_PLACEHOLDERS = ['source', 'build', 'jobs'];

/** Display names of the operating systems of the platform keys. */
const OS_NAMES = { win32: 'Windows', darwin: 'macOS', linux: 'Linux' };

/**
 * One segment of an installed name or a source folder. It cannot start with a dot (so no `.`,
 * `..` or hidden file such as the stamp) nor hold a separator or a drive colon.
 */
const PLAIN_SEGMENT = /^[A-Za-z0-9_+-][A-Za-z0-9._+-]*$/;

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
  const root = /** @type {Record<string, any>} */ (manifest);
  const platforms = root?.platforms;
  if (typeof platforms !== 'object' || platforms === null) {
    throw new Error('the manifest has no "platforms" object');
  }
  for (const field of ['ffmpegVersion', 'whisperVersion', 'whisperLicense']) {
    if (typeof root[field] !== 'string' || root[field].length === 0) {
      problems.push(`${field} must be a non-empty string`);
    }
  }
  for (const key of PLATFORMS) {
    problems.push(...platformProblems(platforms[key]).map((problem) => `${key}${problem}`));
  }
  if (problems.length > 0) {
    throw new Error(`invalid binaries manifest:\n  ${problems.join('\n  ')}`);
  }
  return /** @type {Manifest} */ (manifest);
}

/**
 * Problems of one platform entry, each starting with `: ` or `[index]: `.
 *
 * @param {any} entry
 */
function platformProblems(entry) {
  if (!entry || !Array.isArray(entry.downloads) || entry.downloads.length === 0) {
    return [': no downloads'];
  }
  const problems = [];
  if (typeof entry.license !== 'string' || entry.license.length === 0) {
    problems.push(': no license');
  }
  entry.downloads.forEach((/** @type {any} */ download, /** @type {number} */ index) => {
    problems.push(...downloadProblems(download).map((problem) => `[${index}]: ${problem}`));
  });
  if (problems.length === 0) {
    problems.push(...sharingProblems(entry.downloads).map((problem) => `: ${problem}`));
  }
  return problems;
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
  } else if (!Object.keys(files).every(isPlainRelativePath)) {
    problems.push('installed names must be relative paths of plain segments (e.g. whisper/a.dll)');
  }
  if (download?.build !== undefined) {
    problems.push(...buildProblems(download.build));
  }
  return problems;
}

/** @param {any} build */
function buildProblems(build) {
  const problems = [];
  if (!Object.hasOwn(BUILD_SYSTEMS, build?.system)) {
    problems.push(`build.system must be one of ${Object.keys(BUILD_SYSTEMS).join(', ')}`);
  }
  if (!isPlainRelativePath(build?.source)) {
    problems.push('build.source must be the relative path of the source folder in the archive');
  }
  const steps = build?.steps;
  const isArgv = (/** @type {unknown} */ step) =>
    Array.isArray(step) &&
    step.length > 0 &&
    step.every((argument) => typeof argument === 'string' && argument.length > 0);
  if (!Array.isArray(steps) || steps.length === 0 || !steps.every(isArgv)) {
    problems.push('build.steps must be a non-empty list of argument lists');
  } else {
    const unknown = steps.flat().flatMap((argument) => unknownPlaceholders(argument));
    if (unknown.length > 0) {
      problems.push(
        `build.steps use unknown placeholders ${unknown.join(', ')} ` +
          `(known: ${BUILD_PLACEHOLDERS.map((name) => `{${name}}`).join(', ')})`,
      );
    }
  }
  return problems;
}

/**
 * Two downloads of a platform must not install the same name, nor into the same subfolder: a
 * subfolder belongs to one download, which empties it before installing (stale libraries left
 * there by an older version could be loaded at runtime).
 *
 * @param {Download[]} downloads
 */
function sharingProblems(downloads) {
  const problems = [];
  const owners = new Map();
  for (const [index, download] of downloads.entries()) {
    for (const name of [...Object.keys(download.files), ...installedFolders(download)]) {
      if (owners.has(name) && owners.get(name) !== index) {
        problems.push(`downloads ${owners.get(name)} and ${index} both install ${name}`);
      }
      owners.set(name, index);
    }
  }
  return problems;
}

/**
 * Whether `path` is a relative `/`-separated path of plain segments: not absolute, no empty
 * segment, no `.` or `..`, no backslash or drive colon.
 *
 * @param {unknown} path
 */
export function isPlainRelativePath(path) {
  return (
    typeof path === 'string' && path.split('/').every((segment) => PLAIN_SEGMENT.test(segment))
  );
}

/** @param {string} argument */
function unknownPlaceholders(argument) {
  return [...argument.matchAll(/\{([^}]*)\}/g)]
    .map((match) => match[1] ?? '')
    .filter((name) => !BUILD_PLACEHOLDERS.includes(name))
    .map((name) => `{${name}}`);
}

/**
 * The top-level subfolders a download installs into (`whisper` for `whisper/whisper-cli.exe`).
 *
 * @param {Download} download
 * @returns {string[]}
 */
export function installedFolders(download) {
  const folders = Object.keys(download.files)
    .filter((name) => name.includes('/'))
    .map((name) => name.split('/')[0] ?? name);
  return [...new Set(folders)];
}

/**
 * Human-readable lines describing what will be downloaded for `key`.
 *
 * @param {Manifest} manifest
 * @param {string} key
 * @param {readonly Download[]} [downloads] the downloads to fetch (default: all of the platform)
 * @returns {string[]}
 */
export function describePlan(manifest, key, downloads = manifest.platforms[key]?.downloads ?? []) {
  const entry = manifest.platforms[key];
  const lines = [
    `${key}`,
    `  FFmpeg ${manifest.ffmpegVersion}: ${entry?.license}`,
    `  whisper.cpp ${manifest.whisperVersion}: ${manifest.whisperLicense}`,
  ];
  for (const download of downloads) {
    const note = download.build ? `  (source, compiled here with ${download.build.system})` : '';
    lines.push(`  ${formatSize(download.size).padStart(9)}  ${download.url}${note}`);
  }
  const total = downloads.reduce((sum, download) => sum + download.size, 0);
  lines.push(`  total ${formatSize(total)}`);
  return lines;
}

/**
 * Why `downloads` of platform `key` cannot be prepared on a `hostPlatform` (Node
 * `process.platform`) machine, or `null` if they can: a source build only runs on the platform's
 * own operating system (macOS builds any Mac architecture through `CMAKE_OSX_ARCHITECTURES`).
 *
 * @param {string} key
 * @param {readonly Download[]} downloads
 * @param {string} hostPlatform
 * @returns {string | null}
 */
export function buildHostProblem(key, downloads, hostPlatform) {
  const targetPlatform = key.split('-')[0] ?? key;
  const built = downloads.filter((download) => download.build !== undefined);
  if (targetPlatform === hostPlatform || built.length === 0) {
    return null;
  }
  const name = /** @type {Record<string, string>} */ (OS_NAMES)[targetPlatform] ?? targetPlatform;
  const sources = built.map((download) => download.build?.source).join(', ');
  return (
    `${key} compiles ${sources} from source, which only works on ${name}: ` +
    `run \`pnpm fetch-binaries\` on ${name} to fetch it`
  );
}

/**
 * The commands of a source build, placeholders replaced: `{source}` and `{build}` by the source
 * and build folders, `{jobs}` by the number of parallel compile jobs.
 *
 * @param {Build} build
 * @param {{ source: string, build: string, jobs: number }} values
 * @returns {{ program: string, args: string[] }[]}
 */
export function buildCommands(build, values) {
  const { program } = BUILD_SYSTEMS[build.system];
  const replacements = { source: values.source, build: values.build, jobs: String(values.jobs) };
  return build.steps.map((step) => ({
    program,
    args: step.map((argument) =>
      argument.replace(/\{([^}]*)\}/g, (match, /** @type {string} */ name) =>
        Object.hasOwn(replacements, name)
          ? /** @type {Record<string, string>} */ (replacements)[name]
          : match,
      ),
    ),
  }));
}

/**
 * The downloads of `entry` that the platform folder's stamp does not record (to fetch); empty
 * when the folder is up to date. Each download is checked on its own, so adding one (whisper.cpp
 * next to FFmpeg) only fetches the new one.
 *
 * @param {unknown} stamp
 * @param {PlatformEntry} entry
 * @returns {Download[]}
 */
export function pendingDownloads(stamp, entry) {
  const recorded = /** @type {{ sha256?: unknown }} */ (stamp)?.sha256;
  const installed = new Set(Array.isArray(recorded) ? recorded : []);
  return entry.downloads.filter((download) => !installed.has(download.sha256));
}

/**
 * The stamp of a platform folder holding the downloads of `entry` whose SHA-256 is in `installed`
 * (default: all of them), in manifest order.
 *
 * @param {PlatformEntry} entry
 * @param {ReadonlySet<string>} [installed]
 */
export function stampFor(entry, installed) {
  const hashes = entry.downloads.map((download) => download.sha256);
  return { sha256: installed ? hashes.filter((hash) => installed.has(hash)) : hashes };
}

/** `114.8 MB`. @param {number} bytes */
export function formatSize(bytes) {
  return `${(bytes / 1_000_000).toFixed(1)} MB`;
}

/**
 * A source download is compiled after extraction: each step runs the build system's program in
 * order; the build folder is `build` inside `source`, and `files` name paths in the extracted
 * archive (so the outputs are `<source>/build/...`).
 *
 * @typedef {{ system: keyof typeof BUILD_SYSTEMS, source: string, steps: string[][] }} Build
 * @typedef {{ url: string, sha256: string, size: number, archive: 'zip' | 'tar.xz' | 'tar.gz' | 'none', files: Record<string, string>, build?: Build }} Download
 * @typedef {{ license: string, downloads: Download[] }} PlatformEntry
 * @typedef {{ ffmpegVersion: string, whisperVersion: string, whisperLicense: string, platforms: Record<string, PlatformEntry> }} Manifest
 */
