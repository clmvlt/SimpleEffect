// `pnpm dev`: builds the debug se-core binary, then starts Electron + the renderer dev server.
// Electron main spawns target/debug/se-core itself (see apps/desktop/electron/main/core-binary-path.ts).
import { runSequence } from './lib/run-command.mjs';

await runSequence([
  ['cargo', ['build', '-p', 'se-core']],
  ['pnpm', ['--filter', '@se/desktop', 'dev']],
]);
