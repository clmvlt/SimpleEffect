// `pnpm codegen [--check]`: exports the Rust types deriving `ts_rs::TS` to
// packages/protocol/src/generated/ (ADR-0010, ADR-0012).
//
// The `se-codegen` binary always writes into a scratch folder (`target/codegen/`). Without
// `--check` that folder replaces the committed one; with `--check` the two are compared and the
// script fails if they differ, so a Rust change without `pnpm codegen` fails `pnpm check`.
import { readdir, readFile, rm, cp } from 'node:fs/promises';
import { join, relative, sep } from 'node:path';

import { repoRoot, runCommand } from './lib/run-command.mjs';

const GENERATED_DIR = join(repoRoot, 'packages', 'protocol', 'src', 'generated');
/** Relative to the repository root: `runCommand` arguments must not contain spaces. */
const SCRATCH_RELATIVE = 'target/codegen/generated';
const SCRATCH_DIR = join(repoRoot, ...SCRATCH_RELATIVE.split('/'));

const check = process.argv.includes('--check');

await rm(SCRATCH_DIR, { recursive: true, force: true });
const code = await runCommand('cargo', [
  'run',
  '--quiet',
  '-p',
  'se-codegen',
  '--',
  SCRATCH_RELATIVE,
]);
if (code !== 0) {
  console.error('✖ se-codegen failed (see the error above).');
  process.exit(code);
}

if (check) {
  const problems = await compareTrees(SCRATCH_DIR, GENERATED_DIR);
  if (problems.length > 0) {
    console.error('✖ The generated TypeScript types are stale:');
    for (const problem of problems) {
      console.error(`  - ${problem}`);
    }
    console.error('Run `pnpm codegen` and commit packages/protocol/src/generated/.');
    process.exit(1);
  }
  console.log('✔ Generated TypeScript types are up to date.');
} else {
  await rm(GENERATED_DIR, { recursive: true, force: true });
  await cp(SCRATCH_DIR, GENERATED_DIR, { recursive: true });
  console.log(`✔ Wrote ${relative(repoRoot, GENERATED_DIR)}`);
}

/**
 * Lists the differences between the freshly generated tree and the committed one.
 *
 * Line endings are normalised: Git may check files out with CRLF on Windows.
 *
 * @param {string} freshDir
 * @param {string} committedDir
 * @returns {Promise<string[]>}
 */
async function compareTrees(freshDir, committedDir) {
  const fresh = await listFiles(freshDir);
  const committed = await listFiles(committedDir);
  const problems = [];
  for (const file of fresh) {
    if (!committed.includes(file)) {
      problems.push(`${file} is missing`);
    } else if (
      (await readNormalised(freshDir, file)) !== (await readNormalised(committedDir, file))
    ) {
      problems.push(`${file} differs`);
    }
  }
  for (const file of committed) {
    if (!fresh.includes(file)) {
      problems.push(`${file} is no longer generated`);
    }
  }
  return problems;
}

/**
 * Relative paths (with `/`) of every file under `dir`; empty if `dir` does not exist.
 *
 * @param {string} dir
 * @returns {Promise<string[]>}
 */
async function listFiles(dir) {
  try {
    const entries = await readdir(dir, { recursive: true, withFileTypes: true });
    return entries
      .filter((entry) => entry.isFile())
      .map((entry) => relative(dir, join(entry.parentPath, entry.name)).split(sep).join('/'))
      .sort();
  } catch (error) {
    if (error instanceof Error && 'code' in error && error.code === 'ENOENT') {
      return [];
    }
    throw error;
  }
}

/**
 * @param {string} dir
 * @param {string} file
 */
async function readNormalised(dir, file) {
  const contents = await readFile(join(dir, ...file.split('/')), 'utf8');
  return contents.replaceAll('\r\n', '\n');
}
