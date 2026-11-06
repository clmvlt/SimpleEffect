// `pnpm check`: everything CI runs. Every step runs even if an earlier one fails, so a single run
// reports all problems; the process exits non-zero if any step failed.
import { runCommand } from './lib/run-command.mjs';

/** @type {ReadonlyArray<readonly [string, string, readonly string[]]>} */
const steps = [
  ['Prettier', 'pnpm', ['exec', 'prettier', '--check', '.']],
  ['ESLint', 'pnpm', ['exec', 'eslint', '.', '--max-warnings=0']],
  ['TypeScript', 'pnpm', ['exec', 'tsc', '-b']],
  ['Raw colours', 'node', ['scripts/check-raw-colors.mjs']],
  ['Vitest', 'pnpm', ['exec', 'vitest', 'run']],
  ['Codegen freshness', 'node', ['scripts/codegen.mjs', '--check']],
  ['cargo fmt', 'cargo', ['fmt', '--all', '--check']],
  ['cargo clippy', 'cargo', ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings']],
  ['cargo test', 'cargo', ['test', '--workspace']],
];

const failed = [];
for (const [name, command, args] of steps) {
  console.log(`\n▶ ${name}`);
  const code = await runCommand(command, args);
  if (code !== 0) {
    failed.push(name);
  }
}

if (failed.length > 0) {
  console.error(`\n✖ pnpm check failed: ${failed.join(', ')}`);
  process.exit(1);
}
console.log('\n✔ pnpm check passed');
