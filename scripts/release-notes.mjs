// `node scripts/release-notes.mjs <tag>`: prints the GitHub release notes of <tag> (Conventional
// Commits since the previous tag, grouped, then the installation notes of every platform). Used by
// `.github/workflows/release.yml` (`gh release create --notes-file`); needs the full history
// (`fetch-depth: 0`).
import { execFileSync } from 'node:child_process';

import { releaseNotes } from './lib/release-notes.mjs';
import { repoRoot } from './lib/run-command.mjs';

const tag = process.argv[2];
if (tag === undefined || !/^v\d+\.\d+\.\d+/.test(tag)) {
  console.error('Usage: node scripts/release-notes.mjs <tag>   e.g. v1.2.3');
  process.exit(1);
}

/** @param {string[]} args */
function git(args) {
  return execFileSync('git', args, {
    cwd: repoRoot,
    encoding: 'utf8',
    stdio: ['ignore', 'pipe', 'pipe'],
  }).trim();
}

/** The tag before `current` on its history, `null` for the first release. @param {string} current */
function previousTagOf(current) {
  try {
    return git(['describe', '--tags', '--abbrev=0', `${current}^`]);
  } catch {
    return null;
  }
}

const previousTag = previousTagOf(tag);
const range = previousTag === null ? tag : `${previousTag}..${tag}`;
const commits = git(['log', '--no-merges', '--format=%s%x1f%h', range])
  .split('\n')
  .filter((line) => line !== '')
  .map((line) => {
    const [subject = '', hash = ''] = line.split('\u001f');
    return { subject, hash };
  });

process.stdout.write(releaseNotes({ tag, previousTag, commits }));
