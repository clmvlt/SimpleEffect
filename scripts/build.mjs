// `pnpm build`: release se-core binary + production bundles of the Electron app, then the
// licence notices of what they contain (`scripts/generate-notices.mjs`).
import { rmSync } from 'node:fs';
import { resolve } from 'node:path';

import { repoRoot, runSequence } from './lib/run-command.mjs';

// Stale bundle lists would add packages the new bundles no longer contain.
rmSync(resolve(repoRoot, 'apps/desktop/out/notices'), { recursive: true, force: true });

await runSequence([
  ['cargo', ['build', '--release', '-p', 'se-core']],
  ['pnpm', ['--filter', '@se/desktop', 'build']],
  ['node', ['scripts/generate-notices.mjs']],
]);
