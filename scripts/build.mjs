// `pnpm build`: release se-core binary + production bundles of the Electron app.
import { runSequence } from './lib/run-command.mjs';

await runSequence([
  ['cargo', ['build', '--release', '-p', 'se-core']],
  ['pnpm', ['--filter', '@se/desktop', 'build']],
]);
