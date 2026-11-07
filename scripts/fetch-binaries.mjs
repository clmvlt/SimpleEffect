// Downloads the pinned ffmpeg and ffprobe builds of `scripts/binaries.manifest.json` into
// `resources/bin/<platform>/` (never committed). Shows what it will download (size, source,
// licence) and asks first; `--yes` skips the question (CI), `--platform <key>` / `--all` choose
// other platforms. Every download is checked against its pinned size and SHA-256.
//
// Usage: pnpm fetch-binaries [--yes] [--platform <key>]… [--all]
import { spawn } from 'node:child_process';
import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { chmod, copyFile, mkdir, readdir, readFile, rm, writeFile } from 'node:fs/promises';
import { basename, join, resolve } from 'node:path';
import { createInterface } from 'node:readline/promises';
import { Readable, Transform } from 'node:stream';
import { pipeline } from 'node:stream/promises';

import {
  describePlan,
  isUpToDate,
  parseArgs,
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
  const pending = [];
  for (const key of keys) {
    const stamp = await readJson(join(binRoot, key, STAMP_FILE));
    if (isUpToDate(stamp, manifest.platforms[key])) {
      console.log(`${key}: already up to date (resources/bin/${key}).`);
    } else {
      pending.push(key);
    }
  }
  if (pending.length === 0) {
    return;
  }
  console.log('\nfetch-binaries will download:');
  for (const key of pending) {
    console.log(describePlan(manifest, key).join('\n'));
  }
  console.log(
    '\nThese are third-party GPL builds of FFmpeg for personal use; their sources are linked from' +
      '\nthe builders\x27 pages. Nothing else is downloaded, and the app never downloads at runtime.',
  );
  if (!options.yes && !(await confirm('\nDownload now? [y/N] '))) {
    console.log('Nothing downloaded.');
    return;
  }
  for (const key of pending) {
    await install(key, manifest.platforms[key]);
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
 * Downloads, verifies and extracts the downloads of one platform, then writes its stamp.
 *
 * @param {string} key
 * @param {import('./lib/binaries.mjs').PlatformEntry} entry
 */
async function install(key, entry) {
  const target = join(binRoot, key);
  const work = join(binRoot, `.download-${key}`);
  await rm(work, { recursive: true, force: true });
  await mkdir(work, { recursive: true });
  await mkdir(target, { recursive: true });
  for (const [index, download] of entry.downloads.entries()) {
    const archive = join(work, basename(new URL(download.url).pathname));
    await downloadVerified(download, archive);
    const extracted = join(work, `extracted-${index}`);
    await mkdir(extracted, { recursive: true });
    await extract(archive, download.archive, extracted);
    for (const [name, inner] of Object.entries(download.files)) {
      const source = await locateInArchive(extracted, inner);
      const destination = join(target, name);
      await copyFile(source, destination);
      if (process.platform !== 'win32') {
        await chmod(destination, 0o755);
      }
      console.log(`  installed resources/bin/${key}/${name}`);
    }
  }
  await writeFile(join(target, STAMP_FILE), `${JSON.stringify(stampFor(entry), null, 2)}\n`);
  await rm(work, { recursive: true, force: true });
  console.log(`✔ ${key}: done.`);
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

/** @param {string} file */
async function readJson(file) {
  try {
    return JSON.parse(await readFile(file, 'utf8'));
  } catch {
    return null;
  }
}
