// `pnpm package`: builds everything (`pnpm build`: se-core, bundles, notices), then the
// installer of the current platform with electron-builder (NSIS .exe on Windows, .dmg on macOS,
// .AppImage + .deb on Linux) into apps/desktop/release/.
import { runSequence } from './lib/run-command.mjs';

await runSequence([
  ['node', ['scripts/build.mjs']],
  ['pnpm', ['--filter', '@se/desktop', 'package']],
]);
