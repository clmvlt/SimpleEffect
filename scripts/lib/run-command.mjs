// Cross-platform process helper shared by the repository scripts.
import { spawn } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

/** Absolute path of the repository root. */
export const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');

/**
 * Runs a command with inherited stdio from the repository root and resolves with its exit code.
 *
 * On Windows, `pnpm` and other package-manager tools are `.cmd` shims that only `cmd.exe` can
 * start, so commands go through `cmd.exe /d /s /c` there. Arguments must not contain spaces or
 * quotes (they are not escaped for cmd.exe).
 *
 * @param {string} command
 * @param {readonly string[]} args
 * @param {{ env?: NodeJS.ProcessEnv }} [options]
 * @returns {Promise<number>}
 */
export function runCommand(command, args, options = {}) {
  const [file, fileArgs] =
    process.platform === 'win32'
      ? ['cmd.exe', ['/d', '/s', '/c', command, ...args]]
      : [command, [...args]];
  return new Promise((resolvePromise) => {
    const child = spawn(file, fileArgs, {
      cwd: repoRoot,
      stdio: 'inherit',
      env: { ...process.env, ...options.env },
    });
    child.once('error', (error) => {
      console.error(`Cannot run "${command}": ${error.message}`);
      resolvePromise(1);
    });
    child.once('exit', (code, signal) => resolvePromise(code ?? (signal === null ? 0 : 1)));
  });
}

/**
 * Runs commands one after another and exits the current process at the first failure.
 *
 * @param {ReadonlyArray<readonly [string, readonly string[]]>} commands
 */
export async function runSequence(commands) {
  for (const [command, args] of commands) {
    const code = await runCommand(command, args);
    if (code !== 0) {
      console.error(`\n✖ "${command} ${args.join(' ')}" failed with exit code ${code}.`);
      process.exit(code);
    }
  }
}
