// Downloads the pinned sidecars of `scripts/binaries.manifest.json` into
// `resources/bin/<platform>/` (never committed): ffmpeg and ffprobe, and whisper.cpp in its
// `whisper/` subfolder (compiled locally from the pinned sources on macOS, where there is no
// official build). Shows what it will download (size, source, licence) and asks first; `--yes`
// skips the question (CI), `--platform <key>` / `--all` choose other platforms (`--all` skips the
// ones that must be compiled on another operating system). Every download is checked against its
// pinned size and SHA-256, and only the downloads missing from a platform's stamp are fetched.
//
// Usage: pnpm fetch-binaries [--yes] [--platform <key>]… [--all]
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { chmod, copyFile, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { availableParallelism } from 'node:os';
import { basename, dirname, join, resolve } from 'node:path';
import { createInterface } from 'node:readline/promises';
import { Readable, Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';

import {
  BUILD_SYSTEMS,
  buildCommands,
  buildHostProblem,
  describePlan,
  installedFolders,
  parseArgs,
  pendingDownloads,
  PLATFORMS,
  platformKey,
  STAMP_FILE,
  stampFor,
  validateManifest,
} from './lib/binaries.mjs';
import { repoRoot } from './lib/run-command.mjs';

const binRoot = resolve(repoRoot, 'resources', 'bin');

try {
  await main(process.argv.slice(2));
} catch (error) {
  console.error(`\n✖ fetch-binaries: ${error instanceof Error ? error.message : String(error)}`);
  process.exit(1);
}

/** @param {string[]} argv */
async function main(argv) {
  const options = parseArgs(argv);
  const manifestPath = resolve(repoRoot, 'scripts', 'binaries.manifest.json');
  const manifest = validateManifest(JSON.parse(await readFile(manifestPath, 'utf8')));
  const current = platformKey(process.platform, process.arch);
  const keys = options.all ? PLATFORMS : (options.platforms ?? (current ? [current] : []));
  if (keys.length === 0) {
    throw new Error(`no binaries for ${process.platform}-${process.arch}; use --platform <key>`);
  }
  const plans = await planFetch(manifest, keys, options.all);
  if (plans.length === 0) {
    return;
  }
  await requireBuildTools(plans);
  console.log('\nfetch-binaries will download:');
  for (const plan of plans) {
    console.log(describePlan(manifest, plan.key, plan.downloads).join('\n'));
  }
  console.log(
    '\nFFmpeg: third-party GPL builds, for personal use; their sources are linked from' +
      '\nthe builders\x27 pages. whisper.cpp (MIT): the official ggml-org release builds' +
      '\n(Windows, Linux), or its release sources compiled here with CMake (macOS).' +
      '\nNothing else is downloaded: the app itself downloads the whisper models and the' +
      '\nWindows CUDA build of whisper.cpp only when you ask it to.',
  );
  if (!options.yes && !(await confirm('\nDownload now? [y/N] '))) {
    console.log('Nothing downloaded.');
    return;
  }
  for (const plan of plans) {
    await install(plan);
  }
}

/**
 * The downloads each platform still needs; refuses (or, with `--all`, skips) platforms whose
 * source builds cannot run on this machine.
 *
 * @param {import('./lib/binaries.mjs').Manifest} manifest
 * @param {readonly string[]} keys
 * @param {boolean} all
 * @returns {Promise<Plan[]>}
 */
async function planFetch(manifest, keys, all) {
  const plans = [];
  for (const key of keys) {
    const entry = /** @type {import('./lib/binaries.mjs').PlatformEntry} */ (
      manifest.platforms[key]
    );
    const stamp = await readJson(join(binRoot, key, STAMP_FILE));
    const downloads = pendingDownloads(stamp, entry);
    const problem = buildHostProblem(key, downloads, process.platform);
    if (downloads.length === 0) {
      console.log(`${key}: already up to date (resources/bin/${key}).`);
    } else if (problem === null) {
      plans.push({ key, entry, downloads });
    } else if (all) {
      console.warn(`${key}: skipped, ${problem}.`);
    } else {
      throw new Error(`${problem}. Nothing was downloaded.`);
    }
  }
  return plans;
}

/**
 * Fails before downloading anything when a source build's tool (CMake) is missing.
 *
 * @param {readonly Plan[]} plans
 */
async function requireBuildTools(plans) {
  const builds = plans.flatMap((plan) =>
    plan.downloads.flatMap((download) => download.build ?? []),
  );
  for (const system of new Set(builds.map((build) => build.system))) {
    const { program, hint } = BUILD_SYSTEMS[system];
    if (!(await canRun(program, ['--version']))) {
      const sources = builds
        .filter((build) => build.system === system)
        .map((build) => build.source);
      throw new Error(
        `${program} is needed to compile ${[...new Set(sources)].join(', ')} but was not found: ` +
          `${hint}. Nothing was downloaded.`,
      );
    }
  }
}

/** @param {string} question */
async function confirm(question) {
  if (!process.stdin.isTTY) {
    throw new Error('cannot ask for confirmation in a non-interactive shell: pass --yes');
  }
  const prompt = createInterface({ input: process.stdin, output: process.stdout });
  const answer = await prompt.question(question);
  prompt.close();
  return /^(y|yes|o|oui)$/i.test(answer.trim());
}

/**
 * Installs the pending downloads of one platform, recording each in the stamp once installed (an
 * interrupted run resumes with the downloads it did not finish).
 *
 * @param {Plan} plan
 */
async function install({ key, entry, downloads }) {
  const target = join(binRoot, key);
  const work = join(binRoot, `.download-${key}`);
  await rm(work, { recursive: true, force: true });
  await mkdir(work, { recursive: true });
  await mkdir(target, { recursive: true });
  const pending = new Set(downloads.map((download) => download.sha256));
  const installed = new Set(
    entry.downloads.map((download) => download.sha256).filter((hash) => !pending.has(hash)),
  );
  for (const [index, download] of downloads.entries()) {
    const extracted = join(work, `extracted-${index}`);
    await prepare(download, work, extracted);
    await installFiles(download, extracted, key);
    installed.add(download.sha256);
    const stampText = `${JSON.stringify(stampFor(entry, installed), null, 2)}\n`;
    await writeFile(join(target, STAMP_FILE), stampText);
  }
  await rm(work, { recursive: true, force: true });
  console.log(`✔ ${key}: done.`);
}

/**
 * Downloads, verifies and extracts a download into `extracted`, then compiles it if it is a
 * source download.
 *
 * @param {import('./lib/binaries.mjs').Download} download
 * @param {string} work
 * @param {string} extracted
 */
async function prepare(download, work, extracted) {
  const archive = join(work, basename(new URL(download.url).pathname));
  await downloadVerified(download, archive);
  await mkdir(extracted, { recursive: true });
  await extract(archive, download.archive, extracted);
  if (download.build) {
    await compile(download.build, extracted);
  }
}

/**
 * Copies a download's files into the platform folder. The subfolders it owns are emptied first:
 * libraries of an older version left there could be loaded by the new executable.
 *
 * @param {import('./lib/binaries.mjs').Download} download
 * @param {string} extracted
 * @param {string} key
 */
async function installFiles(download, extracted, key) {
  const target = join(binRoot, key);
  for (const folder of installedFolders(download)) {
    await rm(join(target, folder), { recursive: true, force: true });
  }
  for (const [name, inner] of Object.entries(download.files)) {
    const source = await locateInArchive(extracted, inner);
    const destination = join(target, ...name.split('/'));
    await mkdir(dirname(destination), { recursive: true });
    await copyFile(source, destination);
    if (process.platform !== 'win32') {
      await chmod(destination, 0o755);
    }
    console.log(`  installed resources/bin/${key}/${name}`);
  }
}

/**
 * Compiles an extracted source download with the commands of its manifest `build`.
 *
 * @param {import('./lib/binaries.mjs').Build} build
 * @param {string} extracted
 */
async function compile(build, extracted) {
  const source = join(extracted, ...build.source.split('/'));
  const values = { source, build: join(source, 'build'), jobs: availableParallelism() };
  console.log(`  compiling ${build.source} (this takes a few minutes)`);
  for (const { program, args } of buildCommands(build, values)) {
    console.log(`  > ${program} ${args.join(' ')}`);
    await run(program, args);
  }
}

/**
 * Streams a download to `file`, checking its size and SHA-256.
 *
 * @param {import('./lib/binaries.mjs').Download} download
 * @param {string} file
 */
async function downloadVerified(download, file) {
  console.log(`\nDownloading ${download.url}`);
  const response = await fetch(download.url, { redirect: 'follow' });
  if (!response.ok || response.body === null) {
    throw new Error(`${download.url} answered ${response.status} ${response.statusText}`);
  }
  const hash = createHash('sha256');
  let received = 0;
  let reported = 0;
  const meter = new Transform({
    transform(chunk, _encoding, callback) {
      hash.update(chunk);
      received += chunk.length;
      const percent = Math.floor((received / download.size) * 10) * 10;
      if (percent > reported) {
        reported = percent;
        process.stdout.write(` ${percent}%`);
      }
      callback(null, chunk);
    },
  });
  await pipeline(Readable.fromWeb(response.body), meter, createWriteStream(file));
  process.stdout.write('\n');
  const digest = hash.digest('hex');
  if (received !== download.size || digest !== download.sha256) {
    await rm(file, { force: true });
    throw new Error(
      `${download.url} does not match the manifest (got ${received} bytes, sha256 ${digest}; ` +
        `expected ${download.size} bytes, sha256 ${download.sha256}). Nothing was installed.`,
    );
  }
  console.log('  size and SHA-256 verified');
}

/**
 * Extracts an archive with the system tools: bsdtar reads zip and tar (Windows 10+, macOS),
 * GNU tar reads tar and unzip reads zip (Linux).
 *
 * @param {string} archive
 * @param {'zip' | 'tar.xz' | 'tar.gz' | 'none'} kind
 * @param {string} destination
 */
async function extract(archive, kind, destination) {
  if (kind === 'none') {
    await copyFile(archive, join(destination, basename(archive)));
    return;
  }
  const tar =
    process.platform === 'win32'
      ? join(process.env.SystemRoot ?? 'C:\\Windows', 'System32', 'tar.exe')
      : 'tar';
  if (kind === 'zip' && process.platform === 'linux') {
    await run('unzip', ['-q', '-o', archive, '-d', destination]);
  } else {
    await run(tar, ['-xf', archive, '-C', destination]);
  }
}

/**
 * The extracted file at `inner`, or the only file with its name anywhere in `root` (archive
 * layouts sometimes gain or lose a top folder between releases).
 *
 * @param {string} root
 * @param {string} inner
 */
async function locateInArchive(root, inner) {
  const exact = join(root, ...inner.split('/'));
  const name = basename(exact);
  const entries = await readdir(root, { recursive: true, withFileTypes: true });
  const matches = entries
    .filter((entry) => entry.isFile() && entry.name === name)
    .map((entry) => join(entry.parentPath, entry.name));
  if (matches.includes(exact)) {
    return exact;
  }
  if (matches.length === 1 && matches[0] !== undefined) {
    console.warn(`  note: ${inner} is at ${matches[0]}; update the manifest`);
    return matches[0];
  }
  throw new Error(`${inner} is not in the archive (${matches.length} files named ${name})`);
}

/**
 * Runs a program without a shell and rejects on failure.
 *
 * @param {string} program
 * @param {string[]} args
 */
function run(program, args) {
  return new Promise((resolvePromise, reject) => {
    const child = spawn(program, args, { stdio: 'inherit' });
    child.once('error', (error) => reject(new Error(`cannot run ${program}: ${error.message}`)));
    child.once('exit', (code) =>
      code === 0 ? resolvePromise(undefined) : reject(new Error(`${program} exited with ${code}`)),
    );
  });
}

/**
 * Whether a program starts and exits successfully (used to detect build tools), silently.
 *
 * @param {string} program
 * @param {string[]} args
 * @returns {Promise<boolean>}
 */
function canRun(program, args) {
  return new Promise((resolvePromise) => {
    const child = spawn(program, args, { stdio: 'ignore' });
    child.once('error', () => resolvePromise(false));
    child.once('exit', (code) => resolvePromise(code === 0));
  });
}

/** @param {string} file */
async function readJson(file) {
  try {
    return JSON.parse(await readFile(file, 'utf8'));
  } catch {
    return null;
  }
}

/**
 * What to install in one platform folder: its manifest entry and the downloads it still needs.
 *
 * @typedef {{ key: string, entry: import('./lib/binaries.mjs').PlatformEntry, downloads: import('./lib/binaries.mjs').Download[] }} Plan
 */
