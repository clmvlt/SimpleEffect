// `pnpm package`: builds everything, then the installer of the current platform with
// electron-builder (NSIS .exe on Windows, .dmg on macOS, .AppImage + .deb on Linux) into
// apps/desktop/release/.
import { runSequence } from './lib/run-command.mjs';

await runSequence([
  ['cargo', ['build', '--release', '-p', 'se-core']],
  ['pnpm', ['--filter', '@se/desktop', 'build']],
  ['pnpm', ['--filter', '@se/desktop', 'package']],
]);
